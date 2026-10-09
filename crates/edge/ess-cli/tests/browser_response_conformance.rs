//! Default browser products preserve exact declarations before installation, then execute in Rust.
#[path = "support/browser.rs"]
mod browser;

use ess_cli::TemporaryDirectory;
use std::{fmt::Write as _, fs, path::PathBuf, process::Command, sync::Mutex};

// Each consumer build intentionally shares the authorized compiler cache. Keep artifact copy and
// browser use inside the same lease, so a second installation cannot replace the just-built WASM.
static BUILD_LEASE: Mutex<()> = Mutex::new(());

fn browser_at(evidence: &std::path::Path) -> browser::Browser {
    // Browser receipts belong to the fixture caller; the shared startup contract does not
    // silently create a missing evidence directory before opening its stdout/stderr files.
    fs::create_dir_all(evidence).unwrap();
    browser::Browser::new(evidence)
}

const MODEL: &str = r"format: ess/4
system: demo
version: v1
domain: demo.api
types:
  - name: demo.api.State
    kind: enum
    variants: [Ready, Busy]
  - name: demo.api.Item
    kind: struct
    fields:
      - {name: remaining, type: Integer}
      - {name: created, type: Timestamp}
      - {name: ended, type: Timestamp}
      - {name: state, type: Optional<demo.api.State>}
      - {name: call_type, type: String}
      - {name: features, type: 'Map<String, Boolean>'}
events:
  - name: demo.api.Returned
    fields:
      - {name: item, type: demo.api.Item}
      - {name: receipt, type: String}
commands:
  - name: demo.api.Cancel
    response:
      - {name: item, type: demo.api.Item}
    outcomes:
      - name: cancelled
        emits: [demo.api.Returned]
        payload:
          demo.api.Returned:
            item: {response: item}
            receipt: {generated: true}
";
const AUTHORED: &str = r"type: ess-scenario/4
domain: demo.api
scenario: response-is-observed
summary: The response and emitted event carry the declared item.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.api.Cancel
    outcome: cancelled
    response:
      item:
        remaining: 9007199254740993
        created: 2026-01-05T08:00:00Z
        ended: 2026-01-05T09:00:00Z
        state: null
        call_type: incoming
        features: {recording: true}
    events:
      - event: demo.api.Returned
";

fn emit(route: &str, name: &str) -> TemporaryDirectory {
    emit_sources(route, name, MODEL, AUTHORED)
}
fn emit_sources(route: &str, name: &str, model: &str, authored: &str) -> TemporaryDirectory {
    let root = TemporaryDirectory::create(&format!("ess-browser-product-{name}-{route}")).unwrap();
    fs::write(root.join("system.yaml"), model).unwrap();
    fs::write(root.join("scenario.yaml"), authored).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["verify", "conform", "web", "--path"])
        .arg(root.join("system.yaml"))
        .arg("--scenarios")
        .arg(root.join("scenario.yaml"))
        .args(["--suite-format", route, "--out"])
        .arg(root.join("site"));
    fs::write(root.join("emit.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(root.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(root.join("emit.stderr"), &output.stderr).unwrap();
    fs::write(
        root.join("emit.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    root
}

// These assertions retain the actual baseline defects: successful emission followed by
// ordinary numeric rounding or coverage admission abort. A missing build is not the red.
#[test]
fn ordinary_and_coverage_navigate_exact_response_without_wasm() {
    for route in ["4", "5"] {
        let root = emit(route, "static-response");
        let site = root.join("site");
        assert!(!site.join("runner.wasm").exists());
        let server = browser::Server::new(&site);
        let mut browser = browser_at(&root.join("firefox"));
        let context = browser.open(&format!("{}/index.html", server.url));
        let observed = browser.evaluate(
            &context,
            r"(async () => {
            const until = performance.now() + 10000;
            while (performance.now() < until) {
                if (document.body.dataset.ready || document.body.dataset.error) break;
                await new Promise(resolve => setTimeout(resolve, 20));
            }
            return JSON.stringify({text: document.body.innerText,
                ready: document.body.dataset.ready, error: document.body.dataset.error});
        })()",
        );
        fs::write(root.join("dom.json"), observed.to_string()).unwrap();
        let text = observed["text"].as_str().unwrap();
        assert!(
            text.contains("9007199254740993"),
            "BROWSER_RESPONSE_EXACT_INTEGER_DISPLAY: {observed}"
        );
        assert!(
            !text.contains("9007199254740992"),
            "rounded declaration: {observed}"
        );
        assert_eq!(
            observed["ready"], "true",
            "BROWSER_RESPONSE_COVERAGE_LOAD: {observed}"
        );
        assert!(
            text.contains("build_required"),
            "missing explicit installation state: {observed}"
        );
        assert!(text.contains("Not executed"));
    }
}

#[path = "fixtures/browser-target/src/lib.rs"]
mod independent;

fn load_bundle(
    site: &std::path::Path,
) -> (String, Vec<ess_conformance::web_execution::bundle::Blob>) {
    use ess_conformance::web_execution::bundle::Blob;
    let original = fs::read_to_string(site.join("browser.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&original).unwrap();
    let refs = manifest["sources"]
        .as_array()
        .unwrap()
        .iter()
        .chain([&manifest["execution"]["file"], &manifest["presentation"]]);
    let blobs = refs
        .map(|value| {
            let path = value["path"].as_str().unwrap().to_owned();
            Blob {
                bytes: fs::read(site.join(&path)).unwrap(),
                path,
            }
        })
        .collect();
    (original, blobs)
}
fn native<const MODE: u8>(site: &std::path::Path) -> ess_conformance::web_execution::Completed {
    let (manifest, blobs) = load_bundle(site);
    let factories = independent::factory_calls();
    let mut product =
        ess_conformance::web_execution::Product::<independent::BrowserInstallation<MODE>>::new();
    let handle = product.load(&manifest, blobs).unwrap();
    assert_eq!(
        independent::factory_calls(),
        factories,
        "Load must not install a target"
    );
    product.run(handle, [MODE + 1; 16], 0).unwrap()
}
fn summary(raw: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(raw).unwrap();
    // Compare only actual scenario status/check codes: timestamps and fresh namespace differ.
    serde_json::Value::Array(value["scenarios"].as_array().unwrap().iter().map(|scenario| {
        let checks:Vec<_>=scenario["checks"].as_array().unwrap().iter().map(|check|
            serde_json::json!({"code":check["code"],"status":check["status"]})).collect();
        serde_json::json!({"scenario":scenario["scenario"],"status":scenario["status"],"checks":checks})
    }).collect())
}
fn recorded_command(command: &mut Command, evidence: &std::path::Path, label: &str) {
    fs::write(
        evidence.join(format!("{label}.command")),
        format!("{command:?}\n"),
    )
    .unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join(format!("{label}.stdout")), &output.stdout).unwrap();
    fs::write(evidence.join(format!("{label}.stderr")), &output.stderr).unwrap();
    fs::write(
        evidence.join(format!("{label}.exit")),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn host_build_preflight(root: &std::path::Path, target: &std::path::Path, mode: u8) {
    let Ok(minimum) = std::env::var("ESS_BROWSER_MIN_FREE_BYTES") else {
        return;
    };
    let minimum: u64 = minimum.parse().unwrap();
    fs::create_dir_all(target).unwrap();
    let output = Command::new("df")
        .args(["-B1", "--output=avail"])
        .arg(target)
        .output()
        .unwrap();
    fs::write(root.join(format!("host-{mode}.preflight")), &output.stdout).unwrap();
    assert!(output.status.success(), "disk preflight failed");
    let available: u64 = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(
        available >= minimum,
        "resource refusal: {available} free bytes < {minimum}"
    );
}

fn runtime_source_identity(repo: &std::path::Path, root: &std::path::Path) -> String {
    let saved = root.join("runtime-source.sha256");
    if saved.exists() {
        return fs::read_to_string(saved).unwrap();
    }
    let listed = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(listed.status.success());
    let mut entries = std::collections::BTreeMap::new();
    for path in listed
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        let path = std::str::from_utf8(path).unwrap();
        entries.insert(
            path,
            ess_conformance::web_execution::bundle::hash(&fs::read(repo.join(path)).unwrap()),
        );
    }
    let bytes = serde_json::to_vec(&entries).unwrap();
    let digest = ess_conformance::web_execution::bundle::hash(&bytes);
    fs::write(root.join("runtime-source-files.json"), bytes).unwrap();
    fs::write(saved, &digest).unwrap();
    recorded_command(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo),
        root,
        "runtime-head",
    );
    digest
}

fn build_host(root: &std::path::Path, mode: u8, installation: &str) -> PathBuf {
    let consumer = root.join("consumer");
    fs::create_dir_all(consumer.join("src")).unwrap();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/browser-target");
    let manifest = format!(
        r#"[package]
name = "browser-product-consumer"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["cdylib"]
[workspace]
[dependencies]
ess-conformance = {{ path = {:?} }}
browser-independent-target = {{ path = {:?} }}
[profile.release]
debug = 0
incremental = false
"#,
        repo.join("crates/verify/ess-conformance").to_str().unwrap(),
        fixture.to_str().unwrap()
    );
    fs::write(consumer.join("Cargo.toml"), manifest).unwrap();
    // The emitted module is included unchanged, not replaced by a hand-written test host.
    fs::write(consumer.join("src/lib.rs"),format!("include!({:?});\ninstall_browser_target!(browser_independent_target::{installation}<{mode}>);\n#[unsafe(no_mangle)] pub extern \"C\" fn fixture_factory_calls() -> u32 {{ browser_independent_target::factory_calls() }}\n",root.join("site/rust/browser_host.rs").to_str().unwrap())).unwrap();
    let target = std::env::var_os("ESS_BROWSER_TARGET_DIR")
        .map_or_else(|| root.join("compiled-host"), PathBuf::from);
    if !root.join("rustc-version.stdout").exists() {
        recorded_command(Command::new("rustc").arg("-vV"), root, "rustc-version");
        recorded_command(Command::new("cargo").arg("-V"), root, "cargo-version");
    }
    let runtime_identity = runtime_source_identity(&repo, root);
    let mut lock = Command::new("cargo");
    lock.args(["generate-lockfile", "--offline", "--manifest-path"])
        .arg(consumer.join("Cargo.toml"));
    if !consumer.join("Cargo.lock").exists() {
        recorded_command(&mut lock, root, "host-lock");
    }
    let mut build = Command::new("cargo");
    build
        .args([
            "build",
            "--locked",
            "--offline",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--manifest-path",
        ])
        .arg(consumer.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target)
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-C link-arg=--max-memory=536870912");
    host_build_preflight(root, &target, mode);
    recorded_command(&mut build, root, &format!("host-{mode}"));
    let wasm = target.join("wasm32-unknown-unknown/release/browser_product_consumer.wasm");
    let destination = root.join(format!("runner-{mode}.wasm"));
    fs::copy(wasm, &destination).unwrap();
    let hashes = serde_json::json!({
        "module":ess_conformance::web_execution::bundle::hash(&fs::read(root.join("site/rust/browser_host.rs")).unwrap()),
        "lock":ess_conformance::web_execution::bundle::hash(&fs::read(consumer.join("Cargo.lock")).unwrap()),
        "wasm":ess_conformance::web_execution::bundle::hash(&fs::read(&destination).unwrap()),
        "runtime_source":runtime_identity,
        "fixture_source":ess_conformance::web_execution::bundle::hash(&fs::read(fixture.join("src/lib.rs")).unwrap()),
        "fixture_manifest":ess_conformance::web_execution::bundle::hash(&fs::read(fixture.join("Cargo.toml")).unwrap()),
        "consumer_source":ess_conformance::web_execution::bundle::hash(&fs::read(consumer.join("src/lib.rs")).unwrap()),
        "consumer_manifest":ess_conformance::web_execution::bundle::hash(&fs::read(consumer.join("Cargo.toml")).unwrap()),
    });
    fs::write(
        root.join(format!("host-{mode}.hashes.json")),
        hashes.to_string(),
    )
    .unwrap();
    destination
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn emitted_consumer_module_runs_independent_response_target_in_actual_firefox() {
    let _lease = BUILD_LEASE.lock().unwrap();
    // This is an actual packaging/browser test, not a library-only WASM proxy.
    let authored = format!(
        "{AUTHORED}{}",
        r"        payload:
          item:
            remaining: 9007199254740993
            created: 2026-01-05T08:00:00Z
            ended: 2026-01-05T09:00:00Z
            state: null
            call_type: incoming
            features: {recording: true}
"
    );
    for route in ["4", "5"] {
        let root = emit_sources(route, "executed-response", MODEL, &authored);
        let site = root.join("site");
        let natives = [
            native::<0>(&site),
            native::<1>(&site),
            native::<2>(&site),
            native::<3>(&site),
            native::<4>(&site),
            native::<5>(&site),
            native::<6>(&site),
        ];
        for (mode, native) in [0, 1, 2, 3, 4, 5, 6].into_iter().zip(natives) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "BrowserInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            let result=browser.evaluate(&context,r"(async()=>{
                const until=performance.now()+10000;
                while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
            })()");
            fs::write(
                root.join(format!("executed-{mode}.json")),
                result.to_string(),
            )
            .unwrap();
            let browser_run = result["run"].as_str().unwrap();
            fs::write(root.join(format!("browser-{mode}.run.json")), browser_run).unwrap();
            fs::write(
                root.join(format!("browser-{mode}.report.json")),
                result["report"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                summary(browser_run),
                summary(&native.run),
                "native/browser mode {mode}"
            );
            assert_eq!(result["digest"], native.digest);
            let report: serde_json::Value =
                serde_json::from_str(result["report"].as_str().unwrap()).unwrap();
            assert_eq!(
                report["execution_status"],
                if mode == 0 { "passed" } else { "failed" },
                "{report}"
            );
        }
    }
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn protected_actual_values_never_enter_browser_reports_dom_or_messages() {
    const PROTECTED_MODEL: &str =
        include_str!("../../../verify/ess-conformance/tests/fixtures/one-time-coverage/model.yaml");
    const PROTECTED_SCENARIO: &str = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/one-time-coverage/scenario.yaml"
    );
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = emit_sources(route, "protected", PROTECTED_MODEL, PROTECTED_SCENARIO);
        let site = root.join("site");
        let natives = [
            native_installation::<independent::ProtectedInstallation<0>>(&site, 1),
            native_installation::<independent::ProtectedInstallation<1>>(&site, 2),
            native_installation::<independent::ProtectedInstallation<2>>(&site, 3),
            native_installation::<independent::ProtectedInstallation<3>>(&site, 4),
        ];
        for (mode, native) in [0, 1, 2, 3].into_iter().zip(natives) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "ProtectedInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            browser.subscribe_logs();
            let observed=browser.evaluate(&context,r"(async()=>{
                const until=performance.now()+10000;while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                const messages=[];const OriginalWorker=window.Worker;
                window.Worker=class extends OriginalWorker {constructor(...args){super(...args);this.addEventListener('message',event=>messages.push(event.data));}};
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText,messages},(_key,value)=>value instanceof Uint8Array?new TextDecoder().decode(value):value);
            })()");
            let encoded = observed.to_string();
            fs::write(root.join(format!("protected-{mode}.json")), &encoded).unwrap();
            fs::write(
                root.join(format!("browser-{mode}.report.json")),
                observed["report"].as_str().unwrap(),
            )
            .unwrap();
            fs::write(
                root.join(format!("browser-{mode}.run.json")),
                observed["run"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                summary(observed["run"].as_str().unwrap()),
                summary(&native.run)
            );
            assert_ne!(
                observed["messages"].as_array().unwrap().len(),
                0,
                "actual worker messages must be captured"
            );
            assert!(
                !encoded.contains("private-first-token"),
                "protected value escaped product output"
            );
            assert!(
                !encoded.contains("private-next"),
                "later protected value escaped product output"
            );
            fs::write(root.join(format!("protected-{mode}.json")), &encoded).unwrap();
            let report: serde_json::Value =
                serde_json::from_str(observed["report"].as_str().unwrap()).unwrap();
            assert_eq!(
                report["execution_status"],
                if matches!(mode, 0 | 3) {
                    "passed"
                } else {
                    "failed"
                },
                "{report}"
            );
            // BiDi receipts include the actual DOM/result strings returned by this page.
            let receipt =
                fs::read_to_string(root.join(format!("firefox-{mode}/bidi.jsonl"))).unwrap();
            assert!(!receipt.contains("private-first-token"));
        }
    }
}

fn order_bundle(route: &str) -> TemporaryDirectory {
    let originals = [
        include_str!("../../../../examples/oracle-fixture/system.yaml"),
        include_str!("../../../../examples/oracle-fixture/components.yaml"),
        include_str!("../../../../examples/oracle-fixture/domains/order.yaml"),
        include_str!("../../../../examples/oracle-fixture/domains/dispatch.yaml"),
    ];
    generated_bundle("order", route, &originals)
}

/// Pin the oracle source's known synthesis refusals exactly; every other source refuses nothing.
fn pin_order_refusals(name: &str, refusals: &[ess_conformance::Refusal]) {
    if name == "order" {
        // The existing oracle source intentionally does not publish weight in its views and
        // declares a silent failure policy. Pin those gaps; never discard them or filter scenarios.
        use ess_conformance::{BindingGap, RefusalCause, ScenarioId};
        let mut invariant_cases = Vec::new();
        let mut silent = 0;
        for refusal in refusals {
            match (&refusal.cause, &refusal.scenario) {
                (
                    RefusalCause::InvariantUnobservable {
                        entity,
                        invariant,
                        unpublished,
                        unassertable,
                        state,
                    },
                    Some(ScenarioId::Invariant {
                        entity: scenario_entity,
                        after,
                    }),
                ) => {
                    assert_eq!(entity.to_string(), "oracle.order.Order");
                    assert_eq!(entity, scenario_entity);
                    assert_eq!(invariant, "weight_grams >= 0");
                    assert_eq!(
                        unpublished
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                        ["weight_grams"]
                    );
                    assert_eq!(unassertable.len(), 0, "unassertable fields");
                    invariant_cases.push((
                        after.command.to_string(),
                        after.outcome.to_string(),
                        state.to_string(),
                    ));
                }
                (
                    RefusalCause::BindingUnobservable {
                        binding,
                        gap: BindingGap::PolicySilent,
                    },
                    Some(ScenarioId::Binding {
                        binding: scenario_binding,
                        aspect,
                    }),
                ) => {
                    assert_eq!(binding.to_string(), "handoff-on-shipped");
                    assert_eq!(binding, scenario_binding);
                    assert_eq!(aspect.to_string(), "on-failure");
                    silent += 1;
                }
                _ => panic!("unexpected oracle source refusal: {refusal:?}"),
            }
        }
        assert_eq!(
            invariant_cases,
            [
                (
                    "oracle.order.AmendOrder".into(),
                    "amended".into(),
                    "Placed".into()
                ),
                (
                    "oracle.order.CancelOrder".into(),
                    "cancelled".into(),
                    "Cancelled".into()
                ),
                (
                    "oracle.order.HoldOrder".into(),
                    "held".into(),
                    "Held".into()
                ),
                (
                    "oracle.order.PlaceOrder".into(),
                    "accepted".into(),
                    "Placed".into()
                ),
                (
                    "oracle.order.ShipOrder".into(),
                    "shipped".into(),
                    "Shipped".into()
                ),
            ]
        );
        assert_eq!(silent, 1);
    } else {
        assert_eq!(refusals.len(), 0, "{refusals:?}");
    }
}

fn generated_bundle(name: &str, route: &str, originals: &[&str]) -> TemporaryDirectory {
    use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
    let source_docs: Vec<_> = originals
        .iter()
        .enumerate()
        .map(|(index, text)| SourceDocument {
            path: format!("sources/{index:04}.yaml"),
            text: (*text).into(),
        })
        .collect();
    let parsed = ess_domain::spec::RawSpecFile::parse_all(originals);
    let spec =
        ess_domain::Specification::assemble(parsed.into_iter().enumerate().map(|(index, raw)| {
            (
                ess_domain::system::Source::new(format!("sources/{index:04}.yaml")),
                raw.unwrap(),
            )
        }))
        .unwrap();
    let model = ess_compiler::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
    let synthesis = ess_conformance::synthesize(&model);
    pin_order_refusals(name, &synthesis.refusals);
    let execution = if route == "5" {
        Execution::Coverage(
            ess_conformance::coverage_build::build(
                &model,
                &[],
                ess_conformance::coverage::Scope::System,
                ess_conformance::coverage::Origins::Generated,
            )
            .unwrap(),
        )
    } else {
        Execution::Ordinary(ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap())
    };
    let files = ess_conformance::web::emit_product(&source_docs, &execution).unwrap();
    let root = TemporaryDirectory::create(&format!("ess-browser-{name}-{route}")).unwrap();
    fs::write(
        root.join("synthesis-refusals.txt"),
        format!("{:#?}\n", synthesis.refusals),
    )
    .unwrap();
    for (path, file) in files {
        let path = root.join("site").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, file.contents).unwrap();
    }
    root
}
fn native_order<const MODE: u8>(
    site: &std::path::Path,
) -> ess_conformance::web_execution::Completed {
    let (manifest, blobs) = load_bundle(site);
    let mut product =
        ess_conformance::web_execution::Product::<independent::OrderInstallation<MODE>>::new();
    let handle = product.load(&manifest, blobs).unwrap();
    product.run(handle, [MODE + 1; 16], 0).unwrap()
}
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn full_generated_order_service_exercises_views_grants_lifecycle_and_real_binding_faults() {
    let _lease = BUILD_LEASE.lock().unwrap();
    // The actual default CLI authored routes are covered above. Generated suites use the same
    // complete product emitter and exact host, with no special feature gateway or alternate runner.
    for route in ["4", "5"] {
        let root = order_bundle(route);
        let site = root.join("site");
        let native = [
            native_order::<0>(&site),
            native_order::<1>(&site),
            native_order::<2>(&site),
        ];
        for (mode, native) in [0, 1, 2].into_iter().zip(native) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "OrderInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            let result=browser.evaluate(&context,r"(async()=>{
                const until=performance.now()+10000;while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
            })()");
            fs::write(
                root.join(format!("executed-{mode}.json")),
                result.to_string(),
            )
            .unwrap();
            assert_eq!(
                summary(result["run"].as_str().unwrap()),
                summary(&native.run)
            );
            let report: serde_json::Value =
                serde_json::from_str(result["report"].as_str().unwrap()).unwrap();
            assert_eq!(
                report["execution_status"],
                if mode == 0 { "passed" } else { "failed" },
                "{report}"
            );
            assert!(
                report["counts"]["passed"].as_u64().unwrap() > 0,
                "unrelated scenarios must remain green"
            );
            assert_eq!(report["counts"]["unsupported"], 0);
            assert_eq!(report["counts"]["error"], 0);
            if mode == 1 {
                // A dropped binding is found by eventual checks timing out against the clock the
                // installation supplied, in the worker as natively.
                let run: serde_json::Value =
                    serde_json::from_str(result["run"].as_str().unwrap()).unwrap();
                let eventual = run["scenarios"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|scenario| scenario["checks"].as_array().unwrap())
                    .filter(|check| {
                        check["status"] == "failed" && check["code"] == "ESS-CF-EVENTUAL-EVENT"
                    })
                    .count();
                assert_eq!(eventual, 3, "{run}");
            }
        }
    }
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn generated_nested_observations_compare_independent_response_and_event_in_firefox() {
    const NESTED: &str = r"format: ess/14
system: demo
version: v1
domain: demo.api
types:
  - name: demo.api.Packet
    kind: struct
    fields:
      - {name: value, type: Integer}
events:
  - name: demo.api.Returned
    fields:
      - {name: packet, type: demo.api.Packet}
      - {name: receipt, type: String}
commands:
  - name: demo.api.Read
    response:
      - {name: value, type: Integer}
    outcomes:
      - name: returned
        emits: [demo.api.Returned]
        payload:
          demo.api.Returned:
            packet:
              value: {response: value}
            receipt: {generated: true}
";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = generated_bundle("nested-response", route, &[NESTED]);
        let site = root.join("site");
        let natives = [
            native::<0>(&site),
            native::<1>(&site),
            native::<2>(&site),
            native::<3>(&site),
            native::<4>(&site),
            native::<5>(&site),
        ];
        for (mode, native) in (0..=5).zip(natives) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "BrowserInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            let result = browser.evaluate(&context, r"(async()=>{
                const until=performance.now()+10000;
                while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
            })()");
            fs::write(
                root.join(format!("executed-{mode}.json")),
                result.to_string(),
            )
            .unwrap();
            fs::write(
                root.join(format!("browser-{mode}.report.json")),
                result["report"].as_str().unwrap(),
            )
            .unwrap();
            fs::write(
                root.join(format!("browser-{mode}.run.json")),
                result["run"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                summary(result["run"].as_str().unwrap()),
                summary(&native.run)
            );
            let report: serde_json::Value =
                serde_json::from_str(result["report"].as_str().unwrap()).unwrap();
            assert_eq!(
                report["execution_status"],
                if mode == 0 { "passed" } else { "failed" },
                "{report}"
            );
            assert_eq!(report["counts"]["unsupported"], 0);
            assert_eq!(report["counts"]["error"], 0);
        }
    }
}

fn response_value_forms_source() -> String {
    // Source vocabulary follows support_nested_response/values.rs and the response presence
    // vectors. The installation independently implements its API; it never receives this model.
    let mut source = String::from(
        r"format: ess/15
system: values
version: v1
domain: values.api
types:
  - name: values.api.OptionalPacket
    kind: struct
    fields: [{name: value, type: Optional<Integer>}]
  - name: values.api.Item
    kind: struct
    fields:
      - {name: numbers, type: 'List<Integer>'}
      - {name: note, type: 'Optional<String>'}
  - name: values.api.ListPacket
    kind: struct
    fields: [{name: value, type: 'List<values.api.Item>'}]
events:
  - name: values.api.OptionalReturned
    fields:
      - {name: packet, type: 'Optional<values.api.OptionalPacket>'}
      - {name: receipt, type: String}
  - name: values.api.ListReturned
    fields:
      - {name: packet, type: values.api.ListPacket}
      - {name: receipt, type: String}
commands:
",
    );
    for (command, presence) in [
        ("Absent", ""),
        ("Null", ""),
        ("Equivalent", ""),
        ("Present", ""),
        ("NullPolicy", ", presence: null_when_absent"),
        ("OmitPolicy", ", presence: omitted_when_absent"),
    ] {
        write!(
            source,
            r"  - name: values.api.{command}
    response: [{{name: value, type: 'Optional<Integer>'{presence}}}]
    outcomes:
      - name: returned
        emits: [values.api.OptionalReturned]
        payload:
          values.api.OptionalReturned:
            packet:
              value: {{response: value}}
            receipt: {{generated: true}}
"
        )
        .unwrap();
    }
    source.push_str(
        r"  - name: values.api.Lists
    response: [{name: value, type: 'List<values.api.Item>'}]
    outcomes:
      - name: returned
        emits: [values.api.ListReturned]
        payload:
          values.api.ListReturned:
            packet:
              value: {response: value}
            receipt: {generated: true}
",
    );
    source
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn optional_presence_and_nested_ordered_values_execute_in_both_browser_routes() {
    let _lease = BUILD_LEASE.lock().unwrap();
    let source = response_value_forms_source();
    for route in ["4", "5"] {
        // generated_bundle parses/assembles/compiles these exact source bytes, requires no
        // synthesis refusals, admits the suite/input and emits the complete original product.
        let root = generated_bundle("response-value-forms", route, &[&source]);
        let site = root.join("site");
        let natives = [
            native_installation::<independent::ValueFormsInstallation<0>>(&site, 1),
            native_installation::<independent::ValueFormsInstallation<1>>(&site, 2),
            native_installation::<independent::ValueFormsInstallation<2>>(&site, 3),
            native_installation::<independent::ValueFormsInstallation<3>>(&site, 4),
            native_installation::<independent::ValueFormsInstallation<4>>(&site, 5),
            native_installation::<independent::ValueFormsInstallation<5>>(&site, 6),
            native_installation::<independent::ValueFormsInstallation<6>>(&site, 7),
        ];
        for (mode, native) in (0..=6).zip(natives) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "ValueFormsInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            let result = browser.evaluate(&context, r"(async()=>{
                const until=performance.now()+10000;
                while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
            })()");
            fs::write(
                root.join(format!("executed-{mode}.json")),
                result.to_string(),
            )
            .unwrap();
            let raw_report = result["report"].as_str().unwrap();
            let raw_run = result["run"].as_str().unwrap();
            fs::write(root.join(format!("browser-{mode}.report.json")), raw_report).unwrap();
            fs::write(root.join(format!("browser-{mode}.run.json")), raw_run).unwrap();
            assert_eq!(result["digest"], native.digest);
            assert_eq!(
                raw_report, native.report,
                "complete native/browser report mode {mode}"
            );
            assert_eq!(
                raw_run, native.run,
                "complete native/browser run mode {mode}"
            );
            let report: serde_json::Value = serde_json::from_str(raw_report).unwrap();
            assert_eq!(report["counts"]["total"], 7);
            assert_eq!(report["counts"]["passed"], if mode == 0 { 7 } else { 6 });
            assert_eq!(report["counts"]["failed"], usize::from(mode != 0));
            for count in ["unsupported", "error", "skipped"] {
                assert_eq!(report["counts"][count], 0);
            }
            let run: serde_json::Value = serde_json::from_str(raw_run).unwrap();
            let fault_command = match mode {
                1 => Some("NullPolicy"),
                2 => Some("OmitPolicy"),
                3..=5 => Some("Lists"),
                6 => Some("Present"),
                _ => None,
            };
            let mut ids = Vec::new();
            for scenario in run["scenarios"].as_array().unwrap() {
                let id = scenario["scenario"].as_str().unwrap();
                ids.push(id);
                let fails = fault_command
                    .is_some_and(|command| id == format!("values.api.{command}/outcome/returned"));
                assert_eq!(
                    scenario["status"],
                    if fails { "failed" } else { "passed" },
                    "mode {mode}: {scenario}"
                );
            }
            assert_eq!(
                ids,
                [
                    "values.api.Absent/outcome/returned",
                    "values.api.Equivalent/outcome/returned",
                    "values.api.Lists/outcome/returned",
                    "values.api.Null/outcome/returned",
                    "values.api.NullPolicy/outcome/returned",
                    "values.api.OmitPolicy/outcome/returned",
                    "values.api.Present/outcome/returned",
                ]
            );
        }
    }
}

fn run_frame(id: u32, handle: u32, nonce: u8) -> Vec<u8> {
    let mut frame = b"ESBW".to_vec();
    for word in [1_u32, 0, 3, id, 24, handle] {
        frame.extend_from_slice(&word.to_le_bytes());
    }
    frame.extend_from_slice(&[nonce; 16]);
    frame.extend_from_slice(&0_u32.to_le_bytes());
    frame
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn completed_output_is_cleared_when_coverage_selection_or_runtime_changes() {
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = emit("5", "completed-output-lifecycle");
    let site = root.join("site");
    let wasm = build_host(&root, 0, "BrowserInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox"));
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(&context, r"(async()=>{
        const wait=async predicate=>{const until=performance.now()+10000;while(!predicate()){if(performance.now()>until)throw Error('control timeout');await new Promise(resolve=>setTimeout(resolve,20));}};
        await wait(()=>window.essBrowser);
        const revoked=[],revoke=URL.revokeObjectURL.bind(URL);
        URL.revokeObjectURL=url=>{revoked.push(url);revoke(url);};
        const snapshot=()=>({results:document.getElementById('results').textContent,
            links:[...document.querySelectorAll('#downloads a')].map(a=>a.href),
            provenance:document.getElementById('provenance').textContent,
            state:document.getElementById('runtime-state').textContent,
            moreDisabled:document.getElementById('results-more').disabled,
            backDisabled:document.getElementById('results-back').disabled});
        await window.essBrowser.connect();
        const first=await window.essBrowser.run(),completed=snapshot();
        document.getElementById('select').click();
        await wait(()=>!document.getElementById('run').disabled&&document.getElementById('runtime-state').textContent==='Coverage selection admitted. Not executed.');
        const selected=snapshot(),second=await window.essBrowser.run(),selectedCompleted=snapshot();
        document.getElementById('restore').click();
        await wait(()=>!document.getElementById('run').disabled&&document.getElementById('runtime-state').textContent==='Rust admission complete. Not executed.');
        const restored=snapshot(),third=await window.essBrowser.run(),restoredCompleted=snapshot();
        await window.essBrowser.connect();const reconnected=snapshot();
        const receipt=value=>({digest:value.digest,report:new TextDecoder().decode(value.report),run:new TextDecoder().decode(value.run)});
        return JSON.stringify({first:receipt(first),second:receipt(second),third:receipt(third),completed,selected,selectedCompleted,restored,restoredCompleted,reconnected,revoked,dom:document.body.innerText});
    })()");
    fs::write(
        root.join("completed-output-lifecycle.json"),
        result.to_string(),
    )
    .unwrap();
    assert_ne!(result["first"]["digest"], result["second"]["digest"]);
    assert_eq!(result["first"]["digest"], result["third"]["digest"]);
    let revoked = result["revoked"].as_array().unwrap();
    assert_eq!(
        revoked.len(),
        6,
        "every obsolete download must be revoked exactly once"
    );
    for (run, completed) in [
        ("first", "completed"),
        ("second", "selectedCompleted"),
        ("third", "restoredCompleted"),
    ] {
        assert_ne!(
            result[completed]["results"].as_str().unwrap().len(),
            0,
            "{completed} results"
        );
        let links = result[completed]["links"].as_array().unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().all(|link| revoked.contains(link)));
        let report: serde_json::Value =
            serde_json::from_str(result[run]["report"].as_str().unwrap()).unwrap();
        assert_eq!(report["execution_status"], "passed");
        assert!(result[completed]["provenance"]
            .as_str()
            .unwrap()
            .contains(result[run]["digest"].as_str().unwrap()));
    }
    for cleared in ["selected", "restored", "reconnected"] {
        assert_eq!(
            result[cleared]["results"], "",
            "{cleared} must not retain an earlier run"
        );
        assert_eq!(
            result[cleared]["links"].as_array().unwrap().len(),
            0,
            "{cleared} links"
        );
        assert_eq!(result[cleared]["moreDisabled"], true);
        assert_eq!(result[cleared]["backDisabled"], true);
        assert!(result[cleared]["state"]
            .as_str()
            .unwrap()
            .contains("Not executed."));
    }
}

/// One original-byte mutation the emitted Rust reader must refuse before any factory call.
fn mutate_original(label: &str, bytes: &mut Vec<u8>) {
    let replace = |bytes: &mut Vec<u8>, from: &str, to: &str| {
        *bytes = String::from_utf8(bytes.clone())
            .unwrap()
            .replace(from, to)
            .into_bytes();
    };
    match label {
        "numeric-token" => replace(bytes, "9007199254740993", "9007199254740992"),
        "unknown-execution-member" => bytes
            .splice(1..1, b"\"unknown\":true,".iter().copied())
            .for_each(drop),
        "duplicate-execution-member" => bytes
            .splice(1..1, b"\"duplicate\":1,\"duplicate\":2,".iter().copied())
            .for_each(drop),
        "malformed-utf8" => *bytes = vec![0xff],
        "source-mismatch" => replace(bytes, "version: v1", "version: v2"),
        _ => unreachable!(),
    }
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn actual_emitted_rust_reader_refuses_original_byte_mutations_before_factories() {
    use ess_conformance::web_execution::{abi::load_request, bundle::hash};
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = emit(route, "original-byte-refusal");
        let site = root.join("site");
        let native = native::<0>(&site);
        fs::write(root.join("native.report.json"), &native.report).unwrap();
        fs::write(root.join("native.run.json"), &native.run).unwrap();
        let (manifest, blobs) = load_bundle(&site);
        fs::write(
            site.join("good-load.bin"),
            load_request(1, &manifest, &blobs).unwrap(),
        )
        .unwrap();
        fs::write(site.join("good-run.bin"), run_frame(2, 1, 1)).unwrap();
        fs::write(site.join("stale-run.bin"), run_frame(4, 1, 2)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        let source_count = parsed["sources"].as_array().unwrap().len();
        let mut labels = Vec::new();
        for label in [
            "numeric-token",
            "unknown-execution-member",
            "duplicate-execution-member",
            "malformed-utf8",
            "source-mismatch",
        ] {
            let (_, mut changed) = load_bundle(&site);
            let mut metadata = parsed.clone();
            let index = if label == "source-mismatch" {
                0
            } else {
                source_count
            };
            mutate_original(label, &mut changed[index].bytes);
            assert_ne!(
                changed[index].bytes, blobs[index].bytes,
                "{label} must change original bytes"
            );
            let reference = if index < source_count {
                &mut metadata["sources"][index]
            } else {
                &mut metadata["execution"]["file"]
            };
            reference["sha256"] = hash(&changed[index].bytes).into();
            reference["byte_length"] = changed[index].bytes.len().into();
            fs::write(
                site.join(format!("{label}.bin")),
                load_request(3, &metadata.to_string(), &changed).unwrap(),
            )
            .unwrap();
            labels.push(label);
        }
        fs::write(
            site.join("refusal-cases.json"),
            serde_json::to_vec(&labels).unwrap(),
        )
        .unwrap();
        let wasm = build_host(&root, 0, "BrowserInstallation");
        fs::copy(wasm, site.join("runner.wasm")).unwrap();
        let server = browser::Server::new(&site);
        let mut browser = browser_at(&root.join("firefox"));
        let context = browser.open(&format!("{}/index.html", server.url));
        let result = browser.evaluate(&context, r"(async()=>{
            const module=await WebAssembly.compile(await (await fetch('runner.wasm')).arrayBuffer());
            const read=async path=>new Uint8Array(await (await fetch(path)).arrayBuffer());
            const cases=await (await fetch('refusal-cases.json')).json(),results=[];
            const goodLoad=await read('good-load.bin'),goodRun=await read('good-run.bin'),staleRun=await read('stale-run.bin');
            for(const label of cases){
                const {exports:host}=await WebAssembly.instantiate(module,{});
                let lastResponse;
                const dispatch=frame=>{const at=host.ess_browser_reserve(frame.length);new Uint8Array(host.memory.buffer,at,frame.length).set(frame);const out=host.ess_browser_dispatch(frame.length);lastResponse=new Uint8Array(host.memory.buffer,out,host.ess_browser_response_len()).slice();return new DataView(lastResponse.buffer).getUint32(20,true);};
                const initial=host.fixture_factory_calls();const load=dispatch(goodLoad),afterLoad=host.fixture_factory_calls();
                const run=dispatch(goodRun),afterRun=host.fixture_factory_calls();
                const runResponse=Array.from(lastResponse);
                const refusal=dispatch(await read(label+'.bin')),afterRefusal=host.fixture_factory_calls();
                const stale=dispatch(staleRun),afterStale=host.fixture_factory_calls();
                results.push({label,initial,load,afterLoad,run,afterRun,refusal,afterRefusal,stale,afterStale,memory:host.memory.buffer.byteLength,runResponse});
            }
            return JSON.stringify(results);
        })()");
        fs::write(root.join("refusal-results.json"), result.to_string()).unwrap();
        let dom = browser.evaluate(&context, "JSON.stringify({dom:document.body.innerText})");
        fs::write(root.join("refusal-dom.json"), dom.to_string()).unwrap();
        assert_reader_cases(&root, &result, &native);
    }
}

/// Every refusal leaves the factory count where the healthy run left it; the healthy run's
/// captured linear-memory frame decodes to the exact native report and run bytes.
fn assert_reader_cases(
    root: &std::path::Path,
    result: &serde_json::Value,
    native: &ess_conformance::web_execution::Completed,
) {
    for case in result.as_array().unwrap() {
        assert_eq!(case["initial"], 0, "{case}");
        assert_eq!(case["load"], 0, "{case}");
        assert_eq!(case["afterLoad"], 0, "{case}");
        assert_eq!(case["run"], 0, "{case}");
        assert_eq!(case["afterRun"], 1, "{case}");
        assert_ne!(case["refusal"], 0, "{case}");
        assert_eq!(case["afterRefusal"], 1, "{case}");
        assert_ne!(case["stale"], 0, "{case}");
        assert_eq!(case["afterStale"], 1, "{case}");
        // Decode the captured linear-memory frame in Rust, preserving original report bytes.
        let bytes: Vec<u8> = serde_json::from_value(case["runResponse"].clone()).unwrap();
        let mut offset = 28;
        let text = |offset: &mut usize| {
            let length =
                u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap()) as usize;
            *offset += 4;
            let value = std::str::from_utf8(&bytes[*offset..*offset + length]).unwrap();
            *offset += length;
            value
        };
        assert_eq!(text(&mut offset), native.digest);
        offset += 20; // Exact nonce and generation precede the two original JSON texts.
        let report = text(&mut offset);
        let run = text(&mut offset);
        assert_eq!(report, native.report);
        assert_eq!(run, native.run);
        let label = case["label"].as_str().unwrap();
        fs::write(root.join(format!("browser-{label}.report.json")), report).unwrap();
        fs::write(root.join(format!("browser-{label}.run.json")), run).unwrap();
    }
}

fn native_installation<I: ess_conformance::web_execution::Installation>(
    site: &std::path::Path,
    nonce: u8,
) -> ess_conformance::web_execution::Completed {
    let (manifest, blobs) = load_bundle(site);
    let mut product = ess_conformance::web_execution::Product::<I>::new();
    let handle = product.load(&manifest, blobs).unwrap();
    product.run(handle, [nonce; 16], 0).unwrap()
}

/// Persist both real executions before comparing the complete report/run bytes. The
/// scenario inventory and each fault's expected failed IDs are independent test assertions.
fn stateful_browser_matrix(
    root: &std::path::Path,
    installation: &str,
    natives: Vec<ess_conformance::web_execution::Completed>,
    expected_ids: &[&str],
    failed_ids: &[&[&str]],
) {
    assert_eq!(natives.len(), failed_ids.len());
    let site = root.join("site");
    for (mode, native) in natives.into_iter().enumerate() {
        fs::write(
            root.join(format!("native-{mode}.report.json")),
            &native.report,
        )
        .unwrap();
        fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
        let wasm = build_host(root, u8::try_from(mode).unwrap(), installation);
        fs::copy(wasm, site.join("runner.wasm")).unwrap();
        let server = browser::Server::new(&site);
        let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
        let context = browser.open(&format!("{}/index.html", server.url));
        let result = browser.evaluate(&context, r"(async()=>{
            const until=performance.now()+10000;
            while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            await window.essBrowser.connect();const result=await window.essBrowser.run();
            return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
        })()");
        fs::write(
            root.join(format!("executed-{mode}.json")),
            result.to_string(),
        )
        .unwrap();
        let report_text = result["report"].as_str().unwrap();
        let run_text = result["run"].as_str().unwrap();
        fs::write(
            root.join(format!("browser-{mode}.report.json")),
            report_text,
        )
        .unwrap();
        fs::write(root.join(format!("browser-{mode}.run.json")), run_text).unwrap();
        assert_eq!(result["digest"], native.digest);
        assert_eq!(report_text, native.report, "complete report mode {mode}");
        assert_eq!(run_text, native.run, "complete run mode {mode}");
        let report: serde_json::Value = serde_json::from_str(report_text).unwrap();
        assert_eq!(report["counts"]["total"], expected_ids.len());
        assert_eq!(report["counts"]["failed"], failed_ids[mode].len());
        assert_eq!(
            report["counts"]["passed"],
            expected_ids.len() - failed_ids[mode].len()
        );
        for count in ["unsupported", "error", "skipped"] {
            assert_eq!(report["counts"][count], 0);
        }
        let run: serde_json::Value = serde_json::from_str(run_text).unwrap();
        let scenarios = run["scenarios"].as_array().unwrap();
        let mut actual_ids = Vec::new();
        for scenario in scenarios {
            let id = scenario["scenario"].as_str().unwrap();
            actual_ids.push(id);
            assert_eq!(
                scenario["status"],
                if failed_ids[mode].contains(&id) {
                    "failed"
                } else {
                    "passed"
                },
                "mode {mode}: {scenario}"
            );
        }
        actual_ids.sort_unstable();
        let mut expected_ids = expected_ids.to_vec();
        expected_ids.sort_unstable();
        assert_eq!(actual_ids, expected_ids, "complete scenario inventory");
    }
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn retained_actual_invocation_and_result_replay_execute_in_both_browser_routes() {
    const SOURCE: &str =
        include_str!("../../../verify/ess-conformance/tests/fixtures/retained-replay.yaml");
    const REPLAY: &str = "retained.core.Seed/outcome/replayed";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        // Keep every synthesized scenario, including the original seeded branch. The
        // synthesis helper requires zero refusals; no precomputed reply enters the service.
        let root = generated_bundle("retained-response", route, &[SOURCE]);
        let site = root.join("site");
        let natives = vec![
            native_installation::<independent::RetainedInstallation<0>>(&site, 1),
            native_installation::<independent::RetainedInstallation<1>>(&site, 2),
            native_installation::<independent::RetainedInstallation<2>>(&site, 3),
            native_installation::<independent::RetainedInstallation<3>>(&site, 4),
        ];
        stateful_browser_matrix(
            &root,
            "RetainedInstallation",
            natives,
            &[REPLAY, "retained.core.Seed/outcome/seeded"],
            &[&[], &[REPLAY], &[REPLAY], &[REPLAY]],
        );
    }
}

// The source-owned response identity vector from interpreted_response_values.rs; the
// executable target here is an independent state machine, never the Interpreter.
const CREATION_MODEL: &str = r"format: ess/4
system: demo
version: v1
domain: demo.response
entities:
  - name: demo.response.Row
    identity: {name: id, type: Uuid}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.response.Created
    fields: [{name: id, type: Uuid}]
commands:
  - name: demo.response.Create
    response: [{name: id, type: Uuid}]
    outcomes:
      - name: created
        creates: demo.response.Row
        instance: id
        emits: [demo.response.Created]
        payload: {demo.response.Created: {id: {response: id}}}
views:
  - name: demo.response.CreatedRows
    source: demo.response.Row
    consistency: read_your_writes
    fields: [{name: id, type: Uuid}]
";
const CREATION_AUTHORED: &str = r"type: ess-scenario/4
domain: demo.response
scenario: distinct-response-owned-identities
summary: Repeated creation preserves both response-owned rows and their captured event identities.
arrange:
  - {instance: first, entity: demo.response.Row}
  - {instance: second, entity: demo.response.Row}
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.response.Create
    outcome: created
    response: {}
    events: [{event: demo.response.Created}]
    capture: {instance: first, event: demo.response.Created, field: id}
  - at: 2026-01-05T09:00:01Z
    command: demo.response.Create
    outcome: created
    response: {}
    events: [{event: demo.response.Created}]
    capture: {instance: second, event: demo.response.Created, field: id}
assert:
  - view: demo.response.CreatedRows
    counts: {at_least: 2, at_most: 2}
  - view: demo.response.CreatedRows
    contains: {id: {$instance: first}}
  - view: demo.response.CreatedRows
    contains: {id: {$instance: second}}
";

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn response_owned_creation_identities_remain_distinct_and_correspond_in_both_browser_routes() {
    const GENERATED: &str = "demo.response.Create/outcome/created";
    const AUTHORED: &str = "demo.response/authored/distinct-response-owned-identities";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        for authored in [false, true] {
            // Generated checks prove semantic response/event correspondence; the separate
            // actual CLI-authored two-call scenario proves distinct stored identities. A
            // lone generated create cannot detect a service that reuses its identity later.
            let root = if authored {
                emit_sources(
                    route,
                    "creation-repeated",
                    CREATION_MODEL,
                    CREATION_AUTHORED,
                )
            } else {
                generated_bundle("creation-generated", route, &[CREATION_MODEL])
            };
            let site = root.join("site");
            let natives = vec![
                native_installation::<independent::CreationInstallation<0>>(&site, 1),
                native_installation::<independent::CreationInstallation<1>>(&site, 2),
                native_installation::<independent::CreationInstallation<2>>(&site, 3),
            ];
            if authored {
                stateful_browser_matrix(
                    &root,
                    "CreationInstallation",
                    natives,
                    &[AUTHORED],
                    &[&[], &[AUTHORED], &[AUTHORED]],
                );
            } else {
                stateful_browser_matrix(
                    &root,
                    "CreationInstallation",
                    natives,
                    &[GENERATED],
                    &[&[], &[], &[GENERATED]],
                );
            }
        }
    }
}

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn periodic_host_executes_real_ticks_reads_queue_and_stop_through_emitted_module() {
    const SOURCE: &str = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = generated_bundle("periodic", route, &[SOURCE]);
        let site = root.join("site");
        let natives = [
            native_installation::<independent::PeriodicInstallation<0>>(&site, 1),
            native_installation::<independent::PeriodicInstallation<1>>(&site, 2),
            native_installation::<independent::PeriodicInstallation<2>>(&site, 3),
        ];
        for (mode, native) in (0..=2).zip(natives) {
            fs::write(
                root.join(format!("native-{mode}.report.json")),
                &native.report,
            )
            .unwrap();
            fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
            let wasm = build_host(&root, mode, "PeriodicInstallation");
            fs::copy(wasm, site.join("runner.wasm")).unwrap();
            let server = browser::Server::new(&site);
            let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
            let context = browser.open(&format!("{}/index.html", server.url));
            let result = browser.evaluate(&context, r"(async()=>{
                const until=performance.now()+10000;
                while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
                await window.essBrowser.connect();const result=await window.essBrowser.run();
                return JSON.stringify({report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
            })()");
            fs::write(
                root.join(format!("executed-{mode}.json")),
                result.to_string(),
            )
            .unwrap();
            assert_eq!(
                summary(result["run"].as_str().unwrap()),
                summary(&native.run)
            );
            let run: serde_json::Value =
                serde_json::from_str(result["run"].as_str().unwrap()).unwrap();
            let periodic: Vec<_> = run["scenarios"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|scenario| {
                    scenario["scenario"]
                        .as_str()
                        .unwrap()
                        .starts_with("poll-status/binding/")
                })
                .collect();
            assert_eq!(
                periodic
                    .iter()
                    .map(|scenario| scenario["scenario"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "poll-status/binding/delivery",
                    "poll-status/binding/flow",
                    "poll-status/binding/mapping",
                    "poll-status/binding/on-failure",
                ]
            );
            assert_eq!(
                periodic.len(),
                4,
                "all controlled periodic fixtures must execute"
            );
            for scenario in periodic {
                assert_eq!(
                    scenario["status"],
                    if mode == 0 { "passed" } else { "failed" },
                    "{scenario}"
                );
            }
            let report: serde_json::Value =
                serde_json::from_str(result["report"].as_str().unwrap()).unwrap();
            assert_eq!(
                report["execution_status"],
                if mode == 0 { "passed" } else { "failed" },
                "{report}"
            );
            assert_eq!(report["counts"]["unsupported"], 0);
            assert_eq!(report["counts"]["error"], 0);
        }
    }
}

/// The source each shared one-time vector was derived from (`one_time_contract.rs`).
fn one_time_source(case: &str) -> String {
    const BASE: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
    const CONSTRAINED: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    alphabet: abcdef\n    prefix: ab\n    invariants: [value.count >= 4]\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: credentials.api.Secret}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
    match case {
        "constrained-healthy" | "constrained-invalid" => CONSTRAINED.into(),
        "delayed-event" | "window-unsupported" | "window-short" | "window-healthy" => format!(
            "{BASE}events:\n  - name: credentials.api.Issued\n    fields: [{{name: audit, type: String}}]\n"
        ),
        _ => BASE.into(),
    }
}

fn native_one_time(site: &std::path::Path, mode: u8) -> ess_conformance::web_execution::Completed {
    macro_rules! modes {
        ($($mode:literal)*) => {
            match mode {
                $($mode => native_installation::<independent::OneTimeInstallation<$mode>>(site, $mode + 1),)*
                _ => panic!("no shared one-time mode {mode}"),
            }
        };
    }
    modes!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19)
}

/// Connect and run the served product once in actual Firefox, capturing every worker message.
fn run_capturing_worker_messages(
    site: &std::path::Path,
    evidence: &std::path::Path,
) -> serde_json::Value {
    let server = browser::Server::new(site);
    let mut firefox = browser_at(evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    firefox.subscribe_logs();
    firefox.evaluate(
        &context,
        r"(async()=>{
            const until=performance.now()+10000;while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            const messages=[];const OriginalWorker=window.Worker;
            window.Worker=class extends OriginalWorker {constructor(...args){super(...args);this.addEventListener('message',event=>messages.push(event.data));}};
            await window.essBrowser.connect();const result=await window.essBrowser.run();
            return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText,messages},(_key,value)=>value instanceof Uint8Array?new TextDecoder().decode(value):value);
        })()",
    )
}

/// Every shared one-time observer control (`one-time-execution/manifest.json`, the same exact
/// suite bytes Go and TypeScript execute) runs through the emitted browser product: natively
/// through `Product`, then in actual Firefox through the exact emitted host. Status, all five
/// counts, the fixed rule code and the value-free callback trace equal the manifest; browser and
/// native report/run bytes are identical; no issued plaintext reaches any browser channel.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn shared_one_time_observer_manifest_executes_through_the_emitted_browser_product() {
    use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
    let _lease = BUILD_LEASE.lock().unwrap();
    let vectors = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-conformance/tests/fixtures/one-time-execution");
    let manifest: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(vectors.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.len(), 20, "the complete shared manifest");
    let root = TemporaryDirectory::create("ess-browser-one-time-manifest").unwrap();
    for (index, case) in manifest.iter().enumerate() {
        let mode = u8::try_from(index).unwrap();
        let name = case["case"].as_str().unwrap();
        assert_eq!(independent::one_time_case(mode).as_deref(), Some(name));
        let original = fs::read_to_string(vectors.join(case["suite"].as_str().unwrap())).unwrap();
        let admitted = ess_conformance::AdmittedSuite::from_json(&original).unwrap();
        let files = ess_conformance::web::emit_product(
            &[SourceDocument {
                path: "contract.yaml".into(),
                text: one_time_source(name),
            }],
            &Execution::Ordinary(admitted),
        )
        .unwrap_or_else(|error| panic!("{name}: product emission refused: {error}"));
        let case_root = root.join(name);
        let site = case_root.join("site");
        for (path, file) in files {
            let path = site.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, file.contents).unwrap();
        }
        assert_eq!(
            fs::read_to_string(site.join("suite.json")).unwrap(),
            original,
            "{name}: the product carries the exact shared suite bytes"
        );

        let native = native_one_time(&site, mode);
        assert_eq!(
            serde_json::to_value(independent::last_one_time_trace()).unwrap(),
            case["callback_trace"],
            "{name}: native product callback trace"
        );
        fs::write(case_root.join("native.report.json"), &native.report).unwrap();
        fs::write(case_root.join("native.run.json"), &native.run).unwrap();

        let wasm = build_host(&case_root, mode, "OneTimeInstallation");
        fs::copy(wasm, site.join("runner.wasm")).unwrap();
        let observed = run_capturing_worker_messages(&site, &case_root.join("firefox"));
        let encoded = observed.to_string();
        fs::write(case_root.join("browser.json"), &encoded).unwrap();
        let report_text = observed["report"].as_str().unwrap();
        let run_text = observed["run"].as_str().unwrap();
        fs::write(case_root.join("browser.report.json"), report_text).unwrap();
        fs::write(case_root.join("browser.run.json"), run_text).unwrap();
        assert_eq!(observed["digest"], native.digest, "{name}");
        assert_eq!(report_text, native.report, "{name}: complete report bytes");
        assert_eq!(run_text, native.run, "{name}: complete run bytes");
        assert_ne!(
            observed["messages"].as_array().unwrap().len(),
            0,
            "{name}: actual worker messages must be captured"
        );

        let report: serde_json::Value = serde_json::from_str(report_text).unwrap();
        let mut counts = report["counts"].clone();
        assert_eq!(
            counts.as_object_mut().unwrap().remove("total"),
            Some(serde_json::json!(1)),
            "{name}"
        );
        assert_eq!(counts, case["counts"], "{name}: producer counts");
        let run: serde_json::Value = serde_json::from_str(run_text).unwrap();
        let scenarios = run["scenarios"].as_array().unwrap();
        assert_eq!(scenarios.len(), 1, "{name}");
        assert_eq!(scenarios[0]["status"], case["status"], "{name}: {run}");
        let required = case["required_code"].as_str().unwrap();
        assert!(
            scenarios[0]["checks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|check| check["code"] == required),
            "{name}: {required} missing from {run}"
        );

        let receipt = fs::read_to_string(case_root.join("firefox/bidi.jsonl")).unwrap();
        for channel in [&encoded, &native.report, &native.run, &receipt] {
            for plaintext in ["private-first-token", "fresh-token-"] {
                assert!(
                    !channel.contains(plaintext),
                    "{name}: an issued plaintext escaped a browser channel"
                );
            }
        }
    }
}

/// Execute every installation mode natively through `Product` and in actual Firefox through the
/// exact emitted host. Every scenario's status is named per mode (passed unless listed), the
/// complete native and browser report and run bytes are identical, and the counts add up.
fn status_matrix(
    root: &std::path::Path,
    installation: &str,
    natives: Vec<ess_conformance::web_execution::Completed>,
    expected_ids: &[&str],
    statuses: &[&[(&str, &str)]],
) {
    assert_eq!(natives.len(), statuses.len());
    // Every native status is checked before any module is built, and a mismatch names them all.
    let mut native_statuses = Vec::new();
    let mut mismatched = false;
    for (mode, native) in natives.iter().enumerate() {
        let run: serde_json::Value = serde_json::from_str(&native.run).unwrap();
        for scenario in run["scenarios"].as_array().unwrap() {
            let id = scenario["scenario"].as_str().unwrap().to_owned();
            let status = scenario["status"].as_str().unwrap().to_owned();
            let wanted = statuses[mode]
                .iter()
                .find(|(scenario, _)| *scenario == id)
                .map_or("passed", |(_, status)| *status);
            mismatched |= status != wanted;
            native_statuses.push(format!("mode {mode}: {id} {status} (expected {wanted})"));
        }
    }
    assert!(
        !mismatched,
        "native statuses:\n{}",
        native_statuses.join("\n")
    );
    let site = root.join("site");
    for (mode, native) in natives.into_iter().enumerate() {
        fs::write(
            root.join(format!("native-{mode}.report.json")),
            &native.report,
        )
        .unwrap();
        fs::write(root.join(format!("native-{mode}.run.json")), &native.run).unwrap();
        let expected = |id: &str| {
            statuses[mode]
                .iter()
                .find(|(scenario, _)| *scenario == id)
                .map_or("passed", |(_, status)| *status)
        };
        let native_run: serde_json::Value = serde_json::from_str(&native.run).unwrap();
        let mut native_ids = Vec::new();
        for scenario in native_run["scenarios"].as_array().unwrap() {
            let id = scenario["scenario"].as_str().unwrap();
            native_ids.push(id.to_owned());
            assert_eq!(
                scenario["status"],
                expected(id),
                "native mode {mode}: {scenario}"
            );
        }
        native_ids.sort_unstable();
        let mut wanted: Vec<_> = expected_ids.iter().map(|id| (*id).to_owned()).collect();
        wanted.sort_unstable();
        assert_eq!(native_ids, wanted, "complete scenario inventory");
        let wasm = build_host(root, u8::try_from(mode).unwrap(), installation);
        fs::copy(wasm, site.join("runner.wasm")).unwrap();
        let server = browser::Server::new(&site);
        let mut browser = browser_at(&root.join(format!("firefox-{mode}")));
        let context = browser.open(&format!("{}/index.html", server.url));
        let result = browser.evaluate(&context, r"(async()=>{
            const until=performance.now()+10000;
            while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            await window.essBrowser.connect();const result=await window.essBrowser.run();
            return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),dom:document.body.innerText});
        })()");
        fs::write(
            root.join(format!("executed-{mode}.json")),
            result.to_string(),
        )
        .unwrap();
        let report_text = result["report"].as_str().unwrap();
        let run_text = result["run"].as_str().unwrap();
        fs::write(
            root.join(format!("browser-{mode}.report.json")),
            report_text,
        )
        .unwrap();
        fs::write(root.join(format!("browser-{mode}.run.json")), run_text).unwrap();
        assert_eq!(result["digest"], native.digest);
        assert_eq!(report_text, native.report, "complete report mode {mode}");
        assert_eq!(run_text, native.run, "complete run mode {mode}");
        let report: serde_json::Value = serde_json::from_str(report_text).unwrap();
        assert_eq!(report["counts"]["total"], expected_ids.len());
        for status in ["passed", "failed", "error", "unsupported"] {
            let count = expected_ids
                .iter()
                .filter(|id| expected(id) == status)
                .count();
            assert_eq!(report["counts"][status], count, "mode {mode} {status}");
        }
        assert_eq!(report["counts"]["skipped"], 0);
    }
}

/// The caller-values source (`ess-conformance/tests/caller_values.rs`, beyond10x/ess#168), served
/// over the network with a second actor its grants do not admit (beyond10x/ess#265).
const CALLER_NOTES: &str = "format: ess/16
system: demo
version: v1
summary: Caller-scoped notes with grants.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
  - name: demo.notes.Viewer
    may: []
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: account_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id, text: input.text}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
components:
  - component: notes-service
    owns: {domains: [demo.notes]}
    accepts: {commands: [demo.notes.CreateNote, demo.notes.EditNote]}
    publishes: {events: [demo.notes.NoteCreated, demo.notes.NoteEdited]}
    reached_by: network
";

/// The installation receives the actual actor and caller of every command, keeps its own note
/// rows, decides `forbidden` from the stored agent and refuses actors its grants do not admit.
/// Each fault fails exactly its deciding scenarios, natively and in Firefox, in both routes.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn caller_grants_and_stored_agent_decide_outcomes_through_the_installation_in_both_routes() {
    const CREATED: &str = "demo.notes.CreateNote/outcome/created";
    const FORBIDDEN: &str = "demo.notes.EditNote/outcome/forbidden";
    const EDITED: &str = "demo.notes.EditNote/outcome/edited";
    const CREATE_DENIED: &str = "demo.notes.CreateNote/grant/denied";
    const EDIT_DENIED: &str = "demo.notes.EditNote/grant/denied";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = generated_bundle("caller-grants", route, &[CALLER_NOTES]);
        let site = root.join("site");
        let natives = vec![
            native_installation::<independent::NotesInstallation<0>>(&site, 1),
            native_installation::<independent::NotesInstallation<1>>(&site, 2),
            native_installation::<independent::NotesInstallation<2>>(&site, 3),
            native_installation::<independent::NotesInstallation<3>>(&site, 4),
            native_installation::<independent::NotesInstallation<4>>(&site, 5),
            native_installation::<independent::NotesInstallation<5>>(&site, 6),
        ];
        status_matrix(
            &root,
            "NotesInstallation",
            natives,
            &[CREATED, FORBIDDEN, EDITED, CREATE_DENIED, EDIT_DENIED],
            &[
                &[],
                // The first caller's account is recorded for the second creator too.
                &[(CREATED, "failed"), (EDITED, "failed")],
                &[(FORBIDDEN, "failed")],
                &[(FORBIDDEN, "failed"), (EDITED, "failed")],
                // An actor without grant is refused before authentication; every other send
                // needs a caller this installation cannot authenticate as.
                &[
                    (CREATED, "unsupported"),
                    (FORBIDDEN, "unsupported"),
                    (EDITED, "unsupported"),
                    (EDIT_DENIED, "unsupported"),
                ],
                &[(CREATE_DENIED, "failed"), (EDIT_DENIED, "failed")],
            ],
        );
    }
}

/// Three entity identities that spell alike (`String` "1" beside `Integer` 1, and a `String`
/// spelled as a UUID beside a `Uuid`), established through `EntitySetup` and read back from the
/// installation's own rows. Missing setup, a wrong stored value, an Integer kept as text and one
/// store keyed by spelling each fail the authored scenario.
const RECORDS_MODEL: &str = r"format: ess/1
system: records
version: v1
domain: records.ids
entities:
  - name: records.ids.TextRecord
    identity: {name: record_id, type: String}
    fields:
      - {name: amount, type: Integer}
      - {name: note, type: 'Optional<String>'}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept]}
    invariants:
      - amount >= 0
  - name: records.ids.CountRecord
    identity: {name: record_id, type: Integer}
    fields:
      - {name: amount, type: Integer}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept]}
  - name: records.ids.UuidRecord
    identity: {name: record_id, type: Uuid}
    fields:
      - {name: amount, type: Integer}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept]}
views:
  - name: records.ids.TextRecords
    source: records.ids.TextRecord
    consistency: read_your_writes
    fields:
      - {name: record_id, type: String}
      - {name: amount, type: Integer}
  - name: records.ids.CountRecords
    source: records.ids.CountRecord
    consistency: read_your_writes
    fields:
      - {name: record_id, type: Integer}
      - {name: amount, type: Integer}
  - name: records.ids.UuidRecords
    source: records.ids.UuidRecord
    consistency: read_your_writes
    fields:
      - {name: record_id, type: Uuid}
      - {name: amount, type: Integer}
";
const RECORDS_AUTHORED: &str = r"type: ess-scenario/2
domain: records.ids
scenario: lookalike-identities
summary: Records whose identities spell alike keep their own type, entity and row.
arrange:
  - instance: text-uuid
    entity: records.ids.TextRecord
    setup:
      identity: '00000000-0000-4000-8000-000000000001'
      fields: {amount: 11}
      state: Kept
  - instance: text-one
    entity: records.ids.TextRecord
    setup:
      identity: '1'
      fields: {amount: 12, note: null}
      state: Kept
  - instance: count-one
    entity: records.ids.CountRecord
    setup:
      identity: 1
      fields: {amount: 13}
      state: Kept
  - instance: uuid-one
    entity: records.ids.UuidRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {amount: 14}
      state: Kept
assert:
  - view: records.ids.TextRecords
    contains: {record_id: {$instance: text-uuid}, amount: 11}
  - view: records.ids.TextRecords
    contains: {record_id: {$instance: text-one}, amount: 12}
  - view: records.ids.TextRecords
    counts: {at_least: 2, at_most: 2}
  - view: records.ids.CountRecords
    contains: {record_id: {$instance: count-one}, amount: 13}
  - view: records.ids.CountRecords
    counts: {at_least: 1, at_most: 1}
  - view: records.ids.UuidRecords
    contains: {record_id: {$instance: uuid-one}, amount: 14}
  - view: records.ids.UuidRecords
    counts: {at_least: 1, at_most: 1}
";

#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn entity_setup_rows_and_lookalike_typed_identities_execute_through_the_installation() {
    const SCENARIO: &str = "records.ids/authored/lookalike-identities";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = emit_sources(route, "entity-setup", RECORDS_MODEL, RECORDS_AUTHORED);
        let site = root.join("site");
        let natives = vec![
            native_installation::<independent::SetupInstallation<0>>(&site, 1),
            native_installation::<independent::SetupInstallation<1>>(&site, 2),
            native_installation::<independent::SetupInstallation<2>>(&site, 3),
            native_installation::<independent::SetupInstallation<3>>(&site, 4),
            native_installation::<independent::SetupInstallation<4>>(&site, 5),
        ];
        status_matrix(
            &root,
            "SetupInstallation",
            natives,
            &[SCENARIO],
            &[
                &[],
                &[(SCENARIO, "failed")],
                &[(SCENARIO, "failed")],
                &[(SCENARIO, "failed")],
                &[(SCENARIO, "failed")],
            ],
        );
    }
}

/// The page script fragment that reads a completed run's receipt out of its display document.
const RECEIPT_JS: &str = r"const receipt=display=>Object.fromEntries(display.value.find(([key])=>key==='receipt')[1].value.map(([key,value])=>[key,value.kind==='list'?value.value.map(entry=>entry.value):value.value]));";

/// The current-time guard source (`ess-conformance/tests/fixtures/current-time-guard.yaml`).
const JOBS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/current-time-guard.yaml");

/// Target and Clock are installed together (design section 7): the runner resolves every
/// `now_offset` from the installation's wall clock and the target decides `now - 60s` against the
/// same authority. A target deciding against its own historical epoch, or one running an hour
/// ahead, fails exactly the side of the boundary its clock misplaces.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn installation_clock_shared_with_the_runner_decides_now_guards_in_both_routes() {
    const REFUSAL: &str = "demo.jobs.ScheduleJob/outcome/start-in-past";
    const SCHEDULED: &str = "demo.jobs.ScheduleJob/outcome/scheduled";
    let _lease = BUILD_LEASE.lock().unwrap();
    for route in ["4", "5"] {
        let root = generated_bundle("aligned-clock", route, &[JOBS]);
        let site = root.join("site");
        let natives = vec![
            native_installation::<independent::JobsInstallation<0>>(&site, 1),
            native_installation::<independent::JobsInstallation<1>>(&site, 2),
            native_installation::<independent::JobsInstallation<2>>(&site, 3),
        ];
        status_matrix(
            &root,
            "JobsInstallation",
            natives,
            &[REFUSAL, SCHEDULED],
            &[&[], &[(REFUSAL, "failed")], &[(SCHEDULED, "failed")]],
        );
    }
}

/// Run one product page: connect, run once per entry of `runs` (true: await, false: leave it in
/// flight on `window.pendingRun`), and return each completed receipt and report.
fn run_receipts(browser: &mut browser::Browser, context: &str, runs: usize) -> serde_json::Value {
    browser.evaluate(
        context,
        &format!(
            r"(async()=>{{
            {RECEIPT_JS}
            const until=performance.now()+10000;
            while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            await window.essBrowser.connect();const results=[];
            for(let index=0;index<{runs};index++){{const result=await window.essBrowser.run();
                results.push({{receipt:receipt(result.display),report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run)}});}}
            return JSON.stringify(results);
        }})()"
        ),
    )
}

/// Every Run gets a fresh namespace that `Ids` and the installation share (design section 7):
/// repeated runs in one worker, a reconnected worker, a fresh page and a page running in
/// parallel. A durable store isolating by that namespace stays green on every one of them; the
/// same store isolating by scenario name fails the repeated run in its own worker.
#[allow(clippy::too_many_lines)]
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn fresh_namespaces_isolate_repeated_reconnected_fresh_and_parallel_page_runs() {
    const SCENARIO: &str = "demo.response/authored/distinct-response-owned-identities";
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = emit_sources(
        "4",
        "namespace-isolation",
        CREATION_MODEL,
        CREATION_AUTHORED,
    );
    let site = root.join("site");
    // Natively, one product per installation runs twice, as one worker does.
    let healthy = {
        let (manifest, blobs) = load_bundle(&site);
        let mut product = ess_conformance::web_execution::Product::<
            independent::IsolatedStoreInstallation<0>,
        >::new();
        let handle = product.load(&manifest, blobs).unwrap();
        let first = product.run(handle, [21; 16], 0).unwrap();
        let second = product.run(handle, [22; 16], 0).unwrap();
        [first.report, second.report]
    };
    let faulty_first = {
        let (manifest, blobs) = load_bundle(&site);
        let mut product = ess_conformance::web_execution::Product::<
            independent::IsolatedStoreInstallation<1>,
        >::new();
        let handle = product.load(&manifest, blobs).unwrap();
        let first = product.run(handle, [41; 16], 0).unwrap();
        let second = product.run(handle, [42; 16], 0).unwrap();
        [first.report, second.report]
    };
    let status = |report: &str| -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(report).unwrap()["execution_status"].clone()
    };
    assert_eq!(status(&healthy[0]), "passed");
    assert_eq!(status(&healthy[1]), "passed");
    assert_eq!(status(&faulty_first[0]), "passed");
    assert_eq!(status(&faulty_first[1]), "failed");
    fs::write(root.join("native-healthy.report.json"), &healthy[0]).unwrap();
    fs::write(
        root.join("native-faulty-repeat.report.json"),
        &faulty_first[1],
    )
    .unwrap();

    let wasm = build_host(&root, 0, "IsolatedStoreInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox-healthy"));
    let page = browser.open(&format!("{}/index.html", server.url));
    let repeated = run_receipts(&mut browser, &page, 2);
    let reconnected = run_receipts(&mut browser, &page, 1);
    let fresh = browser.open(&format!("{}/index.html", server.url));
    let parallel = browser.open(&format!("{}/index.html", server.url));
    browser.evaluate(
        &parallel,
        &format!(
            r"(async()=>{{
            {RECEIPT_JS}
            const until=performance.now()+10000;
            while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            window.pendingRun=window.essBrowser.connect().then(()=>window.essBrowser.run()).then(result=>
                ({{receipt:receipt(result.display),report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run)}}));
            return JSON.stringify({{started:true}});
        }})()"
        ),
    );
    let fresh_page = run_receipts(&mut browser, &fresh, 1);
    let parallel_page = browser.evaluate(
        &parallel,
        r"(async()=>JSON.stringify([await window.pendingRun]))()",
    );
    let mut namespaces = std::collections::BTreeSet::new();
    let mut runs = Vec::new();
    for (label, results) in [
        ("repeated", &repeated),
        ("reconnected", &reconnected),
        ("fresh", &fresh_page),
        ("parallel", &parallel_page),
    ] {
        for result in results.as_array().unwrap() {
            let receipt = &result["receipt"];
            let namespace = receipt["namespace"].as_str().unwrap().to_owned();
            assert!(
                namespace.starts_with("browser-")
                    && namespace.ends_with(receipt["selected_digest"].as_str().unwrap()),
                "{label}: {receipt}"
            );
            assert_eq!(receipt["state"], "completed", "{label}: {receipt}");
            assert_eq!(
                result["report"].as_str().unwrap(),
                healthy[0],
                "{label}: complete native/browser report"
            );
            namespaces.insert(namespace);
            runs.push(label);
        }
    }
    fs::write(
        root.join("namespaces.json"),
        serde_json::to_string_pretty(&namespaces).unwrap(),
    )
    .unwrap();
    assert_eq!(runs.len(), 5);
    assert_eq!(
        namespaces.len(),
        5,
        "every run had its own namespace: {namespaces:?}"
    );
    drop(browser);
    drop(server);

    let wasm = build_host(&root, 1, "IsolatedStoreInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox-by-scenario-name"));
    let page = browser.open(&format!("{}/index.html", server.url));
    let faulty = run_receipts(&mut browser, &page, 2);
    let faulty = faulty.as_array().unwrap();
    assert_eq!(faulty[0]["report"].as_str().unwrap(), faulty_first[0]);
    assert_eq!(
        faulty[1]["report"].as_str().unwrap(),
        faulty_first[1],
        "the repeated run reads its earlier rows, natively and in the worker"
    );
    let failed: serde_json::Value =
        serde_json::from_str(faulty[1]["report"].as_str().unwrap()).unwrap();
    assert_eq!(failed["outcomes"]["failed"], serde_json::json!([SCENARIO]));
}

/// Fixtures resolve after admission, inside the run namespace and before their scenario opens,
/// through the same installation (design section 7). A provider resolving them under a shared
/// namespace hands the service a principal it does not own, which the session event exposes.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn fixtures_resolve_in_the_run_namespace_before_their_scenario_opens() {
    const MODEL: &str =
        include_str!("../../../verify/ess-conformance/tests/fixtures/pre-execution-fixtures.yaml");
    const OPENED: &str = "fixturetest.session.Open/outcome/opened";
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = generated_bundle("fixture-namespace", "4", &[MODEL]);
    let site = root.join("site");
    let natives = vec![
        native_installation::<independent::FixtureNamespaceInstallation<0>>(&site, 1),
        native_installation::<independent::FixtureNamespaceInstallation<1>>(&site, 2),
    ];
    status_matrix(
        &root,
        "FixtureNamespaceInstallation",
        natives,
        &[OPENED],
        &[&[], &[(OPENED, "failed")]],
    );
}

/// The synchronous Runner works in a dedicated worker (design section 7): declaration navigation
/// answers while a run is busy; a run completing after navigation moved on is not attached to the
/// new focus; Abort terminates the worker as aborted/cleanup-unconfirmed with no report and no
/// later worker message; a reconnected worker runs again under a fresh namespace.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn worker_runs_keep_navigation_live_detach_stale_completion_and_abort_without_a_report() {
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = emit("4", "worker-scheduling");
    let site = root.join("site");
    let native = native_installation::<independent::SlowInstallation<0>>(&site, 1);
    fs::write(root.join("native.report.json"), &native.report).unwrap();
    let wasm = build_host(&root, 0, "SlowInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox"));
    let context = browser.open(&format!("{}/index.html", server.url));
    let observed = browser.evaluate(
        &context,
        &format!(
            r"(async()=>{{
        {RECEIPT_JS}
        const el=id=>document.getElementById(id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
        const end=performance.now()+10000;while(!window.essBrowser&&performance.now()<end)await sleep(20);
        const messages=[];const OriginalWorker=window.Worker;
        window.Worker=class extends OriginalWorker {{constructor(...args){{super(...args);this.addEventListener('message',event=>messages.push(performance.now()));}}}};
        const panel=()=>({{state:el('runtime-state').textContent,results:el('results').textContent,
            links:document.querySelectorAll('#downloads a').length,runDisabled:el('run').disabled}});
        const decode=result=>({{receipt:receipt(result.display),report:new TextDecoder().decode(result.report)}});
        await window.essBrowser.connect();
        let settled=null;const started=performance.now();
        const first=window.essBrowser.run().then(result=>{{settled=performance.now();return result;}});
        await sleep(300);const steps=[];
        for(let index=0;index<3;index++){{const before=el('position').textContent;el('step').click();await sleep(30);
            steps.push({{at:performance.now(),before,after:el('position').textContent,running:settled===null}});}}
        const stale=decode(await first),detached=panel();
        const second=decode(await window.essBrowser.run()),attached=panel();
        const third=window.essBrowser.run().then(()=>'completed',error=>'rejected:'+error.message);
        await sleep(300);const beforeAbort=messages.length;el('abort').click();
        const aborted=await third;await sleep(1000);const afterAbort=panel();
        const quiet=messages.length-beforeAbort;
        await window.essBrowser.connect();const fourth=decode(await window.essBrowser.run());
        return JSON.stringify({{started,settled,steps,stale,detached,second,attached,aborted,afterAbort,quiet,fourth,dom:document.body.innerText}});
    }})()"
        ),
    );
    fs::write(root.join("scheduling.json"), observed.to_string()).unwrap();
    let steps = observed["steps"].as_array().unwrap();
    assert!(
        steps.iter().all(|step| step["running"] == true),
        "navigation must answer while the worker runs: {observed}"
    );
    assert_ne!(steps[0]["before"], steps[0]["after"], "{observed}");
    assert_eq!(
        observed["detached"]["state"],
        "Completed earlier navigation generation; result not attached to current selection."
    );
    assert_eq!(observed["detached"]["results"], "");
    assert_eq!(observed["detached"]["links"], 0);
    assert_ne!(observed["attached"]["results"], "");
    assert_eq!(observed["attached"]["links"], 2);
    assert_eq!(
        observed["aborted"],
        "rejected:aborted — cleanup unconfirmed"
    );
    assert_eq!(
        observed["afterAbort"]["state"],
        "aborted — cleanup unconfirmed"
    );
    assert_eq!(observed["afterAbort"]["results"], "");
    assert_eq!(observed["afterAbort"]["links"], 0);
    assert_eq!(observed["afterAbort"]["runDisabled"], true);
    assert_eq!(
        observed["quiet"], 0,
        "a terminated worker answered: {observed}"
    );
    let mut namespaces = std::collections::BTreeSet::new();
    for run in ["stale", "second", "fourth"] {
        assert_eq!(
            observed[run]["report"].as_str().unwrap(),
            native.report,
            "{run}: complete native/browser report"
        );
        namespaces.insert(
            observed[run]["receipt"]["namespace"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    assert_eq!(namespaces.len(), 3, "{namespaces:?}");
}

/// Coverage selection three generations deep (design section 1): every report claims exactly the
/// parent it was narrowed from, validates only against its own admitted bytes, and the receipt
/// and the page name the complete retained lineage, nearest first. An ordinary run claims none.
#[allow(clippy::too_many_lines)]
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn three_generation_coverage_reports_claim_only_the_parents_they_have_in_firefox() {
    use ess_conformance::{coverage::AdmittedInput, CountReport};
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = emit("5", "lineage-generations");
    let site = root.join("site");
    let original =
        AdmittedInput::from_json(&fs::read_to_string(site.join("input.json")).unwrap()).unwrap();
    let ids: Vec<_> = original
        .selected()
        .suite()
        .scenarios
        .keys()
        .cloned()
        .collect();
    assert_eq!(ids.len(), 1);
    let mut generations = vec![original];
    for _ in 0..3 {
        let next = generations.last().unwrap().select(&ids).unwrap();
        generations.push(next);
    }
    let (manifest, blobs) = load_bundle(&site);
    let mut product =
        ess_conformance::web_execution::Product::<independent::BrowserInstallation<0>>::new();
    let mut handle = product.load(&manifest, blobs).unwrap();
    let mut natives = Vec::new();
    for generation in 1..=3_u8 {
        handle = product.select(handle, &ids).unwrap();
        natives.push(product.run(handle, [generation; 16], 0).unwrap());
    }
    let wasm = build_host(&root, 0, "BrowserInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox"));
    let context = browser.open(&format!("{}/index.html", server.url));
    let observed = browser.evaluate(
        &context,
        &format!(
            r"(async()=>{{
        {RECEIPT_JS}
        const el=id=>document.getElementById(id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
        const wait=async predicate=>{{const end=performance.now()+20000;while(!predicate()){{if(performance.now()>end)throw Error('control timeout');await sleep(20);}}}};
        await wait(()=>window.essBrowser);await window.essBrowser.connect();const generations=[];
        for(let index=0;index<3;index++){{
            el('select').click();
            await wait(()=>!el('run').disabled&&el('runtime-state').textContent==='Coverage selection admitted. Not executed.');
            const parents=el('provenance').textContent.split('\n').find(line=>line.startsWith('Parents:'));
            const result=await window.essBrowser.run();
            generations.push({{parents,digest:result.digest,receipt:receipt(result.display),
                report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run)}});
        }}
        return JSON.stringify(generations);
    }})()"
        ),
    );
    fs::write(root.join("generations.json"), observed.to_string()).unwrap();
    let observed = observed.as_array().unwrap();
    assert_eq!(observed.len(), 3);
    for (index, (browser_run, native)) in observed.iter().zip(&natives).enumerate() {
        let generation = &generations[index + 1];
        let report_text = browser_run["report"].as_str().unwrap();
        assert_eq!(browser_run["digest"], generation.selected().digest());
        assert_eq!(report_text, native.report, "generation {}", index + 1);
        assert_eq!(browser_run["run"].as_str().unwrap(), native.run);
        let report: serde_json::Value = serde_json::from_str(report_text).unwrap();
        let filter = &report["coverage"]["selection"]["filter"];
        assert_eq!(filter["kind"], "explicit", "{report}");
        assert_eq!(
            filter["parent"]["digest"],
            generations[index].selected().digest(),
            "the report claims exactly its immediate parent"
        );
        let claimed: Vec<_> = generation
            .parents()
            .iter()
            .map(ess_conformance::AdmittedSuite::digest)
            .collect();
        assert_eq!(claimed.len(), index + 1);
        assert_eq!(
            browser_run["receipt"]["parent_digests"],
            serde_json::json!(claimed),
            "the receipt names the complete retained lineage, nearest first"
        );
        // Design section 8: the receipt also binds the original sources, the exact input bytes
        // executed and the installation and runtime that executed them.
        let declared: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(site.join("browser.json")).unwrap()).unwrap();
        let sources: Vec<_> = declared["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| source["sha256"].clone())
            .collect();
        assert_eq!(
            browser_run["receipt"]["source_digests"],
            serde_json::json!(sources)
        );
        assert_eq!(
            browser_run["receipt"]["input_digest"],
            ess_conformance::web_execution::bundle::hash(
                generation
                    .document()
                    .to_canonical_json()
                    .unwrap()
                    .as_bytes()
            )
        );
        assert_eq!(
            browser_run["receipt"]["implementation"],
            report["implementation"]
        );
        assert_eq!(
            browser_run["receipt"]["runtime"],
            ess_conformance::web_execution::RUNTIME
        );
        assert_eq!(
            browser_run["parents"],
            format!("Parents: {}", claimed.join(", ")),
            "the page names the same lineage"
        );
        CountReport::from_json(report_text, generation.selected())
            .expect("the report validates against its own admitted bytes");
        for ancestor in &generations[..=index] {
            assert!(
                CountReport::from_json(report_text, ancestor.selected()).is_err(),
                "a report must not validate as an ancestor's"
            );
        }
    }
    let ordinary = emit("4", "lineage-ordinary");
    let completed = native::<0>(&ordinary.join("site"));
    let report: serde_json::Value = serde_json::from_str(&completed.report).unwrap();
    assert_eq!(
        report["coverage"],
        serde_json::json!({"knowledge": "unknown"})
    );
    let display: serde_json::Value = serde_json::from_str(&completed.display).unwrap();
    assert_eq!(
        display["value"][0][1]["value"][5],
        serde_json::json!(["parent_digests", {"kind": "list", "value": []}])
    );
}

/// An authored scenario naming a command the model does not declare: refused by name, and kept
/// in the declared coverage inventory as a refused authored source.
const REFUSED_AUTHORED: &str = r"type: ess-scenario/4
domain: demo.api
scenario: names-an-undeclared-command
summary: The command named here is not declared, so the scenario is refused by name.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.api.Undeclared
    outcome: cancelled
";

/// The CLI's coverage route (`web --suite-format 5`, authored origins) runs in Firefox, and its
/// report carries exactly the inventory the native CLI declares for the same inputs
/// (`author --suite-format 5`): knowledge, selection, counts and the refused source.
#[allow(clippy::too_many_lines)]
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn coverage_route_reports_the_native_cli_inventory_exactly_in_firefox() {
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = TemporaryDirectory::create("ess-browser-coverage-inventory").unwrap();
    fs::create_dir_all(root.join("scenarios")).unwrap();
    fs::write(root.join("system.yaml"), MODEL).unwrap();
    fs::write(root.join("scenarios/observed.yaml"), AUTHORED).unwrap();
    fs::write(root.join("scenarios/refused.yaml"), REFUSED_AUTHORED).unwrap();
    // Both routes declare the same incomplete inventory and say so the same way: the refused
    // source makes each exit 1 after writing its complete output.
    let exit = |command: &mut Command, label: &str| {
        fs::write(
            root.join(format!("{label}.command")),
            format!("{command:?}\n"),
        )
        .unwrap();
        let output = command.output().unwrap();
        fs::write(root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        fs::write(
            root.join(format!("{label}.exit")),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
        output.status.code()
    };
    let native_exit = exit(
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .args([
                "verify",
                "conform",
                "author",
                "--suite-format",
                "5",
                "--path",
            ])
            .arg(root.join("system.yaml"))
            .arg("--scenarios")
            .arg(root.join("scenarios"))
            .arg("--out")
            .arg(root.join("native-suite.json")),
        "native-inventory",
    );
    let web_exit = exit(
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["verify", "conform", "web", "--suite-format", "5", "--path"])
            .arg(root.join("system.yaml"))
            .arg("--scenarios")
            .arg(root.join("scenarios"))
            .arg("--out")
            .arg(root.join("site")),
        "web-inventory",
    );
    assert_eq!((native_exit, web_exit), (Some(1), Some(1)));
    let site = root.join("site");
    let native_suite = fs::read_to_string(root.join("native-suite.json")).unwrap();
    let native_suite = ess_conformance::AdmittedSuite::from_json(&native_suite).unwrap();
    let product_input = ess_conformance::coverage::AdmittedInput::from_json(
        &fs::read_to_string(site.join("input.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        product_input.selected().original_json(),
        native_suite.original_json(),
        "the product carries the native CLI's exact coverage suite bytes"
    );
    let inventory = native_suite.coverage().unwrap();
    let native = native::<0>(&site);
    let wasm = build_host(&root, 0, "BrowserInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let mut browser = browser_at(&root.join("firefox"));
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(&context, r"(async()=>{
        const until=performance.now()+10000;
        while(!window.essBrowser&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
        await window.essBrowser.connect();const result=await window.essBrowser.run();
        return JSON.stringify({digest:result.digest,report:new TextDecoder().decode(result.report),run:new TextDecoder().decode(result.run),provenance:document.getElementById('provenance').textContent});
    })()");
    fs::write(root.join("executed.json"), result.to_string()).unwrap();
    let report_text = result["report"].as_str().unwrap();
    assert_eq!(result["digest"], native.digest);
    assert_eq!(report_text, native.report, "complete native/browser report");
    assert_eq!(result["run"].as_str().unwrap(), native.run);
    let report: serde_json::Value = serde_json::from_str(report_text).unwrap();
    let expected = serde_json::json!({
        "knowledge": inventory.knowledge,
        "selection": inventory.selection,
        "counts": inventory.counts,
        "refused": inventory.refused,
    });
    assert_eq!(
        report["coverage"], expected,
        "the report's inventory is the CLI's"
    );
    assert_eq!(report["coverage"]["knowledge"], "complete_inventory");
    assert_eq!(report["coverage"]["selection"]["origins"], "authored");
    assert_eq!(report["coverage"]["counts"]["authored"], 1);
    assert_eq!(report["coverage"]["refused"].as_array().unwrap().len(), 1);
    assert_eq!(report["counts"]["total"], 1);
    assert_eq!(report["counts"]["passed"], 1);
    assert_ne!(
        report["conformance_status"], "passed",
        "a refused source keeps the conformance claim from passing"
    );
    let provenance = result["provenance"].as_str().unwrap();
    assert!(provenance.contains("complete_inventory"), "{provenance}");
}

/// Section 9: `--suite-format` keeps its spelling and selects the product kind; the page states
/// the version of the suite actually emitted, read from the suite, never the option label.
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn suite_format_selects_the_product_kind_and_the_page_shows_the_emitted_suite_version() {
    let mut versions = Vec::new();
    for (route, kind, file) in [
        ("4", "ordinary_suite", "suite.json"),
        ("5", "coverage_input", "input.json"),
    ] {
        let root = emit(route, "suite-format-label");
        let site = root.join("site");
        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(site.join("browser.json")).unwrap()).unwrap();
        assert_eq!(manifest["execution"]["kind"], kind);
        assert_eq!(manifest["execution"]["file"]["path"], file);
        let original = fs::read_to_string(site.join(file)).unwrap();
        let version = if route == "4" {
            ess_conformance::AdmittedSuite::from_json(&original)
                .unwrap()
                .suite()
                .provenance
                .suite_version
                .to_string()
        } else {
            ess_conformance::coverage::AdmittedInput::from_json(&original)
                .unwrap()
                .selected()
                .suite()
                .provenance
                .suite_version
                .to_string()
        };
        let server = browser::Server::new(&site);
        let mut browser = browser_at(&root.join("firefox"));
        let context = browser.open(&format!("{}/index.html", server.url));
        let observed = browser.evaluate(
            &context,
            r"(async()=>{
            const until=performance.now()+10000;
            while(!document.body.dataset.ready&&!document.body.dataset.error&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            return JSON.stringify({ready:document.body.dataset.ready??null,provenance:document.getElementById('provenance').textContent,text:document.body.innerText});
        })()",
        );
        fs::write(root.join("suite-format.json"), observed.to_string()).unwrap();
        assert_eq!(observed["ready"], "true", "{observed}");
        let provenance = observed["provenance"].as_str().unwrap();
        assert!(
            provenance.contains(&format!("\"{version}\"")),
            "route {route}: the page must state the emitted suite version {version}: {provenance}"
        );
        assert!(
            !provenance.contains(&format!("ess-conformance/{route}\"")),
            "route {route}: the option label is not a suite major: {provenance}"
        );
        versions.push(version);
    }
    assert_ne!(
        versions[0], versions[1],
        "the coverage route selects a coverage suite version"
    );
}

/// Section 9: the historical replay/1 reader and emitter stay explicit legacy routes with their
/// exact asset bytes and refusals; the default product carries none of their files and accepts
/// what the legacy emitter refuses.
#[allow(clippy::too_many_lines)]
#[test]
fn legacy_replay_emitters_stay_separate_byte_stable_and_refusing() {
    const NESTED: &str = r"format: ess/14
system: demo
version: v1
domain: demo.api
types:
  - name: demo.api.Packet
    kind: struct
    fields:
      - {name: value, type: Integer}
events:
  - name: demo.api.Returned
    fields:
      - {name: packet, type: demo.api.Packet}
      - {name: receipt, type: String}
commands:
  - name: demo.api.Read
    response:
      - {name: value, type: Integer}
    outcomes:
      - name: returned
        emits: [demo.api.Returned]
        payload:
          demo.api.Returned:
            packet:
              value: {response: value}
            receipt: {generated: true}
";
    use ess_conformance::web_execution::bundle::hash;
    let root = emit("4", "legacy-separation");
    let site = root.join("site");
    for legacy in [
        "model.json",
        "replay.json",
        "admission.js",
        "coverage-player.js",
    ] {
        assert!(
            !site.join(legacy).exists(),
            "default product wrote {legacy}"
        );
    }
    let legacy_site = root.join("legacy");
    fs::create_dir_all(&legacy_site).unwrap();
    for (path, file) in fs::read_dir(&site)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .map(|path| (path.file_name().unwrap().to_owned(), path))
    {
        fs::copy(file, legacy_site.join(path)).unwrap();
    }
    copy_tree(&site.join("sources"), &legacy_site.join("sources"));
    browser::legacy_replay_fixture(&legacy_site);
    for (name, pinned) in [
        (
            "index.html",
            "cf1c6e121de499b64e26d27febf5c8f18fad53bd6031ee98e02dde80d26b9240",
        ),
        (
            "player.js",
            "0bc7077dba992b563414dd47661bd90af5e4e0fdde17337897ffd4c66da70916",
        ),
    ] {
        assert_eq!(
            hash(&fs::read(legacy_site.join(name)).unwrap()),
            pinned,
            "legacy {name} bytes"
        );
    }
    assert!(legacy_site.join("model.json").exists());
    let coverage = emit("5", "legacy-separation-coverage");
    let coverage_site = coverage.join("legacy");
    fs::create_dir_all(&coverage_site).unwrap();
    for entry in fs::read_dir(coverage.join("site")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, coverage_site.join(path.file_name().unwrap())).unwrap();
        }
    }
    copy_tree(
        &coverage.join("site/sources"),
        &coverage_site.join("sources"),
    );
    browser::legacy_replay_fixture(&coverage_site);
    for (name, pinned) in [
        (
            "player.js",
            "70d21a41663f11f5d3e2203ebb36731c1d9d79047bcdd14c203440a53b742834",
        ),
        (
            "admission.js",
            "7258f1b7335fe95a94188138a0295c909577c1498a63d31499bbb957085398b8",
        ),
    ] {
        assert_eq!(
            hash(&fs::read(coverage_site.join(name)).unwrap()),
            pinned,
            "legacy coverage {name} bytes"
        );
    }
    // A response-mapping suite: the legacy emitter keeps refusing it; the default product emits it.
    let nested = generated_bundle("legacy-refusal", "4", &[NESTED]);
    let nested_suite = ess_conformance::AdmittedSuite::from_json(
        &fs::read_to_string(nested.join("site/suite.json")).unwrap(),
    )
    .unwrap();
    let parsed = ess_domain::spec::RawSpecFile::parse_all(&[NESTED]);
    let spec = ess_domain::Specification::assemble(parsed.into_iter().map(|raw| {
        (
            ess_domain::system::Source::new("sources/0000.yaml"),
            raw.unwrap(),
        )
    }))
    .unwrap();
    let model = ess_compiler::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
    assert!(
        ess_conformance::web::emit(&model, nested_suite.suite()).is_err(),
        "legacy replay/1 keeps refusing response mappings"
    );
    assert!(nested.join("site/browser.json").exists());
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, &target).unwrap();
        }
    }
}

/// Section 9: the browser product registers product, presentation and ABI identities only. Its
/// reports are the existing report format, its suites are the admitted original bytes, and the
/// three identities are the ones the format catalog registers.
#[test]
fn the_browser_product_registers_no_new_suite_or_report_version() {
    let catalog = include_str!("../../../../docs/design/review-format-catalog.md");
    for identity in [
        "ess-conformance-browser/1",
        "ess-conformance-browser-presentation/1",
        "ess-conformance-browser-abi/1",
    ] {
        assert!(catalog.contains(identity), "{identity} is not registered");
    }
    assert_eq!(
        ess_conformance::web_execution::bundle::FORMAT,
        "ess-conformance-browser/1"
    );
    assert_eq!(
        ess_conformance::web_execution::presentation::FORMAT,
        "ess-conformance-browser-presentation/1"
    );
    assert_eq!(
        ess_conformance::web_execution::ABI,
        "ess-conformance-browser-abi/1"
    );
    for route in ["4", "5"] {
        let root = emit(route, "registered-identities");
        let site = root.join("site");
        let completed = native::<0>(&site);
        let report: serde_json::Value = serde_json::from_str(&completed.report).unwrap();
        let file = if route == "4" {
            "suite.json"
        } else {
            "input.json"
        };
        let original = fs::read_to_string(site.join(file)).unwrap();
        let admitted = if route == "4" {
            ess_conformance::AdmittedSuite::from_json(&original).unwrap()
        } else {
            ess_conformance::coverage::AdmittedInput::from_json(&original)
                .unwrap()
                .selected()
                .clone()
        };
        let native_report = ess_conformance::CountReport::from_json(&completed.report, &admitted)
            .expect("the existing report reader admits the product's report");
        assert_eq!(
            native_report.to_canonical_json().unwrap(),
            completed.report,
            "the report is the existing canonical report"
        );
        assert_eq!(
            report["suite"]["version"],
            admitted.suite().provenance.suite_version.to_string()
        );
    }
}

/// Section 10's required validation, family by family: each family names the executed tests that
/// carry its evidence, and the part of a family no test executes is named here rather than counted.
/// Every named test must exist as a `#[test]` without `#[ignore]` in the file named beside it.
#[allow(clippy::too_many_lines)]
#[test]
fn section_ten_validation_names_an_executed_test_for_every_family() {
    const CLI: &str = include_str!("browser_response_conformance.rs");
    const ADVERSARY: &str = include_str!("browser_product_adversary.rs");
    const ADMISSION: &str =
        include_str!("../../../verify/ess-conformance/tests/browser_product_admission.rs");
    const PRESENTATION: &str =
        include_str!("../../../verify/ess-conformance/tests/browser_product_presentation.rs");
    const CONFORMANCE_ADVERSARY: &str =
        include_str!("../../../verify/ess-conformance/tests/browser_product_adversary.rs");
    // (family, executed tests as (source, name), parts no test executes yet)
    type Family<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a [&'a str]);
    let families: &[Family] = &[
        (
            "Responses",
            &[
                (CLI, "emitted_consumer_module_runs_independent_response_target_in_actual_firefox"),
                (CLI, "generated_nested_observations_compare_independent_response_and_event_in_firefox"),
                (CLI, "optional_presence_and_nested_ordered_values_execute_in_both_browser_routes"),
                (CLI, "retained_actual_invocation_and_result_replay_execute_in_both_browser_routes"),
            ],
            &["stale-result control for a response observed after its scenario closed"],
        ),
        (
            "Protected policies and creation identity",
            &[
                (CLI, "protected_actual_values_never_enter_browser_reports_dom_or_messages"),
                (CLI, "shared_one_time_observer_manifest_executes_through_the_emitted_browser_product"),
                (CLI, "response_owned_creation_identities_remain_distinct_and_correspond_in_both_browser_routes"),
            ],
            &[],
        ),
        (
            "Original bytes and lineage",
            &[
                (CLI, "actual_emitted_rust_reader_refuses_original_byte_mutations_before_factories"),
                (CLI, "three_generation_coverage_reports_claim_only_the_parents_they_have_in_firefox"),
                (ADMISSION, "full_coverage_lineage_survives_selection_and_missing_parent_refuses_before_installation"),
                (CONFORMANCE_ADVERSARY, "forged_multi_level_lineage_refuses_before_factory_after_a_prior_run"),
            ],
            &[],
        ),
        (
            "Ordinary/coverage distinction",
            &[
                (CONFORMANCE_ADVERSARY, "ordinary_kind_never_admits_coverage_suite_bytes_without_their_carrier"),
                (CONFORMANCE_ADVERSARY, "coverage_kind_never_wraps_an_ordinary_suite"),
                (ADMISSION, "no_ordinary_coverage_fabrication_and_handles_do_not_rebind"),
                (CLI, "coverage_route_reports_the_native_cli_inventory_exactly_in_firefox"),
            ],
            &[],
        ),
        (
            "Full declaration fidelity",
            &[
                (CLI, "ordinary_and_coverage_navigate_exact_response_without_wasm"),
                (PRESENTATION, "numeric_payloads_are_text_and_original_suite_bytes_never_round_trip_through_display"),
            ],
            &["a per-declaration-kind card check for preserves/deletes/sets/affects/aggregates"],
        ),
        (
            "Scheduling and selection",
            &[
                (CLI, "completed_output_is_cleared_when_coverage_selection_or_runtime_changes"),
                (CLI, "worker_runs_keep_navigation_live_detach_stale_completion_and_abort_without_a_report"),
            ],
            &[],
        ),
        (
            "Isolation and clocks",
            &[
                (CLI, "fresh_namespaces_isolate_repeated_reconnected_fresh_and_parallel_page_runs"),
                (CLI, "fixtures_resolve_in_the_run_namespace_before_their_scenario_opens"),
                (CLI, "installation_clock_shared_with_the_runner_decides_now_guards_in_both_routes"),
                (CLI, "periodic_host_executes_real_ticks_reads_queue_and_stop_through_emitted_module"),
                // Its dropped-binding fault fails as ESS-CF-EVENTUAL-EVENT: the eventual check times
                // out against the installed clock.
                (CLI, "full_generated_order_service_exercises_views_grants_lifecycle_and_real_binding_faults"),
            ],
            &[],
        ),
        (
            "Every other admitted Runner capability",
            &[
                (CLI, "caller_grants_and_stored_agent_decide_outcomes_through_the_installation_in_both_routes"),
                (CLI, "entity_setup_rows_and_lookalike_typed_identities_execute_through_the_installation"),
                (CLI, "full_generated_order_service_exercises_views_grants_lifecycle_and_real_binding_faults"),
                (CLI, "retained_actual_invocation_and_result_replay_execute_in_both_browser_routes"),
            ],
            &[
                "no-input and no-op commands",
                "related-entity and set effects",
                "binding retries and delivery context",
                "ordered scans (halt)",
                "clock readings",
            ],
        ),
        (
            "ABI, paths and resources",
            &[
                (ADMISSION, "bounded_abi_admits_original_load_and_rejects_buffer_misuse"),
                (ADMISSION, "nonces_are_consumed_on_factory_failure_and_capacity_never_evicts"),
                (ADMISSION, "resource_profile_admits_each_exact_bound_and_refuses_one_over"),
                (CLI, "wasm_frame_memory_and_fetch_budgets_are_enforced_and_measured_in_firefox"),
            ],
            &[],
        ),
        (
            "Disclosure",
            &[
                (ADVERSARY, "adversary_trap_and_factory_panic_never_disclose_or_report"),
                (ADVERSARY, "adversary_abort_of_a_running_target_is_not_reported_as_internal_failure"),
                (ADVERSARY, "adversary_factory_error_ends_visibly_without_report_or_disclosure"),
                (ADVERSARY, "adversary_watchdog_timeout_ends_cleanup_unconfirmed_without_report_or_disclosure"),
                (ADVERSARY, "adversary_admission_refusal_after_an_executed_secret_discloses_nothing"),
            ],
            &[],
        ),
        (
            "Legacy and neighboring products",
            &[
                (CLI, "legacy_replay_emitters_stay_separate_byte_stable_and_refusing"),
                (CLI, "suite_format_selects_the_product_kind_and_the_page_shows_the_emitted_suite_version"),
                (CLI, "the_browser_product_registers_no_new_suite_or_report_version"),
            ],
            &[],
        ),
    ];
    assert_eq!(families.len(), 11, "every section 10 family is listed");
    let mut not_executed = Vec::new();
    for (family, tests, gaps) in families {
        assert_ne!(tests.len(), 0, "{family} names no executed test");
        for (source, name) in *tests {
            let declared = format!("#[test]\nfn {name}(");
            assert!(
                source.contains(&declared),
                "{family}: `{name}` is not a test in the file named"
            );
            let at = source.find(&declared).unwrap();
            assert!(
                !source[..at]
                    .lines()
                    .rev()
                    .take(4)
                    .any(|line| line.contains("#[ignore")
                        && !line.contains("browser-product lane: `task test-browser-product`")),
                "{family}: `{name}` is ignored outside the browser-product lane"
            );
        }
        not_executed.extend(gaps.iter().map(|gap| format!("{family}: {gap}")));
    }
    // What no browser test executes yet is a named gap, never counted as coverage.
    assert_eq!(
        not_executed,
        [
            "Responses: stale-result control for a response observed after its scenario closed",
            "Full declaration fidelity: a per-declaration-kind card check for preserves/deletes/sets/affects/aggregates",
            "Every other admitted Runner capability: no-input and no-op commands",
            "Every other admitted Runner capability: related-entity and set effects",
            "Every other admitted Runner capability: binding retries and delivery context",
            "Every other admitted Runner capability: ordered scans (halt)",
            "Every other admitted Runner capability: clock readings",
        ]
    );
}

/// Section 4 in actual Firefox: the emitted module reserves exactly the 64 MiB frame and refuses
/// one byte more; a Load frame of exactly that size reaches bundle admission; linear memory grows
/// to exactly 8,192 pages and no further; measured high-water marks are retained. The static page
/// counts actual fetched bytes against the manifest and refuses an over-long path label as a
/// resource limit before any fetch, while an exactly 1,024-byte label loads.
#[allow(clippy::too_many_lines)]
#[ignore = "browser-product lane: `task test-browser-product` (slower than a CI test shard)"]
#[test]
fn wasm_frame_memory_and_fetch_budgets_are_enforced_and_measured_in_firefox() {
    use ess_conformance::web_execution::{abi::load_request, MAX_FRAME, MAX_PATH};
    let _lease = BUILD_LEASE.lock().unwrap();
    let root = emit("4", "resource-profile");
    let site = root.join("site");
    let native = native::<0>(&site);
    let (manifest, blobs) = load_bundle(&site);
    let good = load_request(1, &manifest, &blobs).unwrap();
    fs::write(site.join("good-load.bin"), &good).unwrap();
    fs::write(site.join("good-run.bin"), run_frame(2, 1, 7)).unwrap();
    // Exactly MAX_FRAME bytes: the declared blobs, the last one zero-padded to fill the frame.
    let mut padded = blobs.clone();
    let fill = MAX_FRAME - good.len();
    padded
        .last_mut()
        .unwrap()
        .bytes
        .extend(std::iter::repeat_n(0, fill));
    let exact = load_request(2, &manifest, &padded).unwrap();
    assert_eq!(exact.len(), MAX_FRAME);
    fs::write(site.join("exact-load.bin"), &exact).unwrap();
    // A valid bundle whose Load frame is exactly MAX_FRAME: the original source padded with one
    // YAML comment line; model, suite and provenance stay the emitted ones.
    let emitted_suite = ess_conformance::AdmittedSuite::from_json(
        &fs::read_to_string(site.join("suite.json")).unwrap(),
    )
    .unwrap();
    let padded_frame = |pad: usize| {
        use ess_conformance::web_execution::bundle::{create, Execution, SourceDocument};
        let (manifest, blobs) = create(
            &[SourceDocument {
                path: "sources/0000.yaml".into(),
                text: format!("{MODEL}#{}\n", "p".repeat(pad)),
            }],
            &Execution::Ordinary(emitted_suite.clone()),
        )
        .unwrap();
        load_request(3, &manifest, &blobs).unwrap()
    };
    // From this probe to the exact size every length field keeps eight digits, so the frame
    // grows by exactly one byte per padding byte.
    let probe = 16 * 1024 * 1024;
    let pad = probe + MAX_FRAME - padded_frame(probe).len();
    let valid = padded_frame(pad);
    assert_eq!(valid.len(), MAX_FRAME);
    fs::write(site.join("valid-exact-load.bin"), &valid).unwrap();
    fs::write(site.join("valid-exact-run.bin"), run_frame(4, 1, 9)).unwrap();
    let wasm = build_host(&root, 0, "BrowserInstallation");
    fs::copy(wasm, site.join("runner.wasm")).unwrap();

    // Static budget sites: an exactly 1,024-byte label, one byte more, and a blob longer than its
    // declared length.
    let label = {
        let mut label = String::from("sources");
        while label.len() + 201 + ".yaml".len() <= MAX_PATH {
            label.push('/');
            label.push_str(&"d".repeat(200));
        }
        label.push('/');
        label.push_str(&"f".repeat(MAX_PATH - label.len() - ".yaml".len()));
        label.push_str(".yaml");
        label
    };
    assert_eq!(label.len(), MAX_PATH);
    let exact_label = root.join("exact-label");
    {
        use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
        let files = ess_conformance::web::emit_product(
            &[SourceDocument {
                path: label.clone(),
                text: MODEL.into(),
            }],
            &Execution::Ordinary(emitted_suite.clone()),
        )
        .unwrap();
        for (path, file) in files {
            let path = exact_label.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, file.contents).unwrap();
        }
        let over = ess_conformance::web::emit_product(
            &[SourceDocument {
                path: format!("{label}x"),
                text: MODEL.into(),
            }],
            &Execution::Ordinary(emitted_suite.clone()),
        );
        assert_eq!(
            over.err(),
            Some(ess_conformance::web_execution::Error::ResourceLimit),
            "Rust emission refuses a 1,025-byte label as a resource limit"
        );
    }
    let over_label = root.join("over-label");
    copy_tree(&exact_label, &over_label);
    let manifest_text = fs::read_to_string(over_label.join("browser.json")).unwrap();
    fs::write(
        over_label.join("browser.json"),
        manifest_text.replace(&label, &format!("{label}x")),
    )
    .unwrap();
    let over_blob = root.join("over-blob");
    copy_tree(&site, &over_blob);
    let mut suite_bytes = fs::read(over_blob.join("suite.json")).unwrap();
    suite_bytes.push(b'\n');
    fs::write(over_blob.join("suite.json"), suite_bytes).unwrap();

    let mut browser = browser_at(&root.join("firefox"));
    let mut pages = serde_json::Map::new();
    for (name, directory) in [
        ("exact-label", &exact_label),
        ("over-label", &over_label),
        ("over-blob", &over_blob),
    ] {
        let server = browser::Server::new(directory);
        let context = browser.open(&format!("{}/index.html", server.url));
        let observed = browser.evaluate(
            &context,
            r"(async()=>{
            const until=performance.now()+10000;
            while(!document.body.dataset.ready&&!document.body.dataset.error&&performance.now()<until)await new Promise(resolve=>setTimeout(resolve,20));
            return JSON.stringify({ready:document.body.dataset.ready??null,error:document.getElementById('error').textContent});
        })()",
        );
        pages.insert(name.into(), observed);
    }
    let server = browser::Server::new(&site);
    let context = browser.open(&format!("{}/index.html", server.url));
    let measured = browser.evaluate(
        &context,
        &format!(
            r"(async()=>{{
        const MAX={MAX_FRAME};
        const module=await WebAssembly.compile(await (await fetch('runner.wasm')).arrayBuffer());
        const read=async path=>new Uint8Array(await (await fetch(path)).arrayBuffer());
        const fresh=async()=>(await WebAssembly.instantiate(module,{{}})).exports;
        const pages=host=>host.memory.buffer.byteLength/65536;
        const dispatch=(host,frame)=>{{const at=host.ess_browser_reserve(frame.length);if(!at)return {{reserved:false}};
            new Uint8Array(host.memory.buffer,at,frame.length).set(frame);const out=host.ess_browser_dispatch(frame.length);
            const response=new Uint8Array(host.memory.buffer,out,host.ess_browser_response_len()).slice(),view=new DataView(response.buffer);
            return {{reserved:true,tag:view.getUint32(12,true),status:view.getUint32(20,true)}};}};
        const result={{}};
        let host=await fresh();result.initialPages=pages(host);
        result.reserveExact=host.ess_browser_reserve(MAX)!==0;result.afterReservePages=pages(host);
        result.reserveOver=host.ess_browser_reserve(MAX+1)!==0;
        result.exactLoad=dispatch(host,await read('exact-load.bin'));result.afterExactLoadPages=pages(host);
        host=await fresh();
        result.load=dispatch(host,await read('good-load.bin'));result.afterLoadPages=pages(host);
        result.run=dispatch(host,await read('good-run.bin'));result.afterRunPages=pages(host);
        host=await fresh();
        try{{result.validExactLoad=dispatch(host,await read('valid-exact-load.bin'));result.afterValidExactLoadPages=pages(host);
            result.validExactRun=dispatch(host,await read('valid-exact-run.bin'));result.afterValidExactRunPages=pages(host);}}
        catch(error){{result.validExactTrap=error.name;result.afterValidExactLoadPages=pages(host);}}
        host=await fresh();const before=pages(host);
        result.growToMaximum=host.memory.grow(8192-before)===before;result.maximumPages=pages(host);
        try{{host.memory.grow(1);result.growOver='grew';}}catch(error){{result.growOver=error.name;}}
        return JSON.stringify(result);
    }})()"
        ),
    );
    pages.insert("measured".into(), measured.clone());
    let receipt = serde_json::Value::Object(pages);
    fs::write(root.join("resource-profile.json"), receipt.to_string()).unwrap();
    assert_eq!(receipt["exact-label"]["ready"], "true", "{receipt}");
    assert_eq!(
        receipt["over-label"]["error"], "resource_limit",
        "{receipt}"
    );
    assert_eq!(receipt["over-blob"]["error"], "resource_limit", "{receipt}");
    assert_eq!(measured["reserveExact"], true, "{measured}");
    assert_eq!(measured["reserveOver"], false, "{measured}");
    // The exact frame passes the frame layer and is refused by bundle admission, not by a bound.
    assert_eq!(measured["exactLoad"]["tag"], 5, "{measured}");
    assert_eq!(
        measured["exactLoad"]["status"],
        ess_conformance::web_execution::Error::InvalidBundle as u32,
        "{measured}"
    );
    assert_eq!(measured["load"]["tag"], 1, "{measured}");
    assert_eq!(measured["run"]["tag"], 3, "{measured}");
    // The largest valid Load the profile advertises is admitted and runs inside linear memory.
    assert_eq!(measured["validExactLoad"]["tag"], 1, "{measured}");
    assert_eq!(measured["validExactRun"]["tag"], 3, "{measured}");
    assert_eq!(measured["growToMaximum"], true, "{measured}");
    assert_eq!(measured["maximumPages"], 8192, "{measured}");
    assert_eq!(measured["growOver"], "RangeError", "{measured}");
    for key in [
        "afterReservePages",
        "afterExactLoadPages",
        "afterRunPages",
        "afterValidExactLoadPages",
        "afterValidExactRunPages",
    ] {
        assert!(
            measured[key].as_u64().unwrap() <= 8192,
            "{key} exceeded the linear-memory maximum: {measured}"
        );
    }
    let native_report: serde_json::Value = serde_json::from_str(&native.report).unwrap();
    assert_eq!(native_report["execution_status"], "passed");
}
