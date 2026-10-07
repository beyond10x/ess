//! Default browser products preserve exact declarations before installation, then execute in Rust.
#[path = "support/browser.rs"]
mod browser;

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

fn emit(route: &str, name: &str) -> PathBuf {
    emit_sources(route, name, MODEL, AUTHORED)
}
fn emit_sources(route: &str, name: &str, model: &str, authored: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ess-browser-product-{name}-{route}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
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

fn order_bundle(route: &str) -> PathBuf {
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

fn generated_bundle(name: &str, route: &str, originals: &[&str]) -> PathBuf {
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
    let root =
        std::env::temp_dir().join(format!("ess-browser-{name}-{route}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
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
        }
    }
}

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
#[test]
fn shared_one_time_observer_manifest_executes_through_the_emitted_browser_product() {
    use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
    let _lease = BUILD_LEASE.lock().unwrap();
    let vectors = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-conformance/tests/fixtures/one-time-execution");
    let manifest: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(vectors.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.len(), 20, "the complete shared manifest");
    let root = std::env::temp_dir().join(format!(
        "ess-browser-one-time-manifest-{}",
        std::process::id()
    ));
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
