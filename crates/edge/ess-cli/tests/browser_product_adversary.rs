//! Adversary cases for the emitted browser product's error paths in actual Firefox: a target trap
//! holding an issued secret, a panicking factory, and an explicit Abort of a running target.
#[path = "support/browser.rs"]
mod browser;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const SENTINEL: &str = "adv-sentinel-4f1c9e2a7b";
const SOURCE: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";

fn emit(root: &Path) -> PathBuf {
    use ess_conformance::web_execution::bundle::{Execution, SourceDocument};
    let original = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/one-time-execution/healthy.json"
    );
    let admitted = ess_conformance::AdmittedSuite::from_json(original).unwrap();
    let files = ess_conformance::web::emit_product(
        &[SourceDocument {
            path: "contract.yaml".into(),
            text: SOURCE.into(),
        }],
        &Execution::Ordinary(admitted),
    )
    .unwrap();
    let site = root.join("site");
    for (path, file) in files {
        let path = site.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, file.contents).unwrap();
    }
    site
}

fn run(command: &mut Command, root: &Path, label: &str) {
    let output = command.output().unwrap();
    fs::write(root.join(format!("{label}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{label}.stderr")), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Build the exact emitted `rust/browser_host.rs` against the adversary installation.
fn build_host(root: &Path, mode: u8) -> PathBuf {
    let consumer = root.join(format!("consumer-{mode}"));
    fs::create_dir_all(consumer.join("src")).unwrap();
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime = manifest_dir
        .join("../../verify/ess-conformance")
        .canonicalize()
        .unwrap();
    let fixture = manifest_dir.join("tests/fixtures/adversary-browser-target");
    fs::write(
        consumer.join("Cargo.toml"),
        format!(
            "[package]\nname = \"adversary-consumer\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[lib]\ncrate-type = [\"cdylib\"]\n[workspace]\n[dependencies]\ness-conformance = {{ path = {:?} }}\nadversary-browser-target = {{ path = {:?} }}\n[profile.release]\ndebug = 0\nincremental = false\n",
            runtime.to_str().unwrap(),
            fixture.to_str().unwrap()
        ),
    )
    .unwrap();
    fs::write(
        consumer.join("src/lib.rs"),
        format!(
            "include!({:?});\ninstall_browser_target!(adversary_browser_target::AdversaryInstallation<{mode}>);\n",
            root.join("site/rust/browser_host.rs").to_str().unwrap()
        ),
    )
    .unwrap();
    run(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(consumer.join("Cargo.toml")),
        root,
        &format!("lock-{mode}"),
    );
    let target = std::env::var_os("ESS_BROWSER_TARGET_DIR")
        .map_or_else(|| root.join("compiled-host"), PathBuf::from);
    run(
        Command::new("cargo")
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
            .env("RUSTFLAGS", "-C link-arg=--max-memory=536870912"),
        root,
        &format!("host-{mode}"),
    );
    target.join("wasm32-unknown-unknown/release/adversary_consumer.wasm")
}

/// Connect, press Run, optionally press Abort, and return every page channel the test can read.
const FLOW: &str = r"(async()=>{
    const el=id=>document.getElementById(id);
    const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
    const until=async(predicate,ms)=>{const end=performance.now()+ms;while(!predicate()&&performance.now()<end)await sleep(20);return predicate();};
    await until(()=>window.essBrowser,10000);
    const messages=[];const OriginalWorker=window.Worker;
    window.Worker=class extends OriginalWorker {constructor(...args){super(...args);this.addEventListener('message',event=>messages.push(event.data));}};
    await window.essBrowser.connect();
    const loaded=el('runtime-state').textContent;
    el('run').click();
    let during=null;
    if(ABORT){
        await sleep(1500);
        during={state:el('runtime-state').textContent,abort:el('abort').disabled,run:el('run').disabled};
        el('abort').click();
    }
    await until(()=>el('runtime-state').textContent!==loaded&&el('error').textContent!=='',30000);
    await sleep(300);
    return JSON.stringify({loaded,during,state:el('runtime-state').textContent,error:el('error').textContent,
        results:el('results').textContent,links:document.querySelectorAll('#downloads a').length,
        runDisabled:el('run').disabled,dom:document.body.innerText,messages},
        (_key,value)=>value instanceof Uint8Array?new TextDecoder().decode(value):value);
})()";

fn observe(root: &Path, site: &Path, mode: u8, abort: bool) -> (serde_json::Value, String) {
    let wasm = build_host(root, mode);
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(site);
    let evidence = root.join(format!("firefox-{mode}"));
    fs::create_dir_all(&evidence).unwrap();
    let mut firefox = browser::Browser::new(&evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    firefox.subscribe_logs();
    let observed = firefox.evaluate(
        &context,
        &FLOW.replace("ABORT", if abort { "true" } else { "false" }),
    );
    drop(firefox);
    let encoded = observed.to_string();
    fs::write(root.join(format!("observed-{mode}.json")), &encoded).unwrap();
    let receipt = fs::read_to_string(evidence.join("bidi.jsonl")).unwrap();
    (observed, format!("{encoded}\n{receipt}"))
}

fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ess-browser-adversary-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

/// A trap in the target after the runner received an issued secret, and a factory panic carrying
/// one, end as a visible execution error with no report and no plaintext on any page channel.
#[test]
fn adversary_trap_and_factory_panic_never_disclose_or_report() {
    let root = root("trap");
    let site = emit(&root);
    for mode in [0, 1] {
        let (observed, channels) = observe(&root, &site, mode, false);
        assert!(
            !channels.contains(SENTINEL),
            "mode {mode}: an issued plaintext escaped a browser channel"
        );
        assert_eq!(
            observed["state"], "execution_error",
            "mode {mode}: {observed}"
        );
        assert_eq!(
            observed["error"], "execution_error",
            "mode {mode}: {observed}"
        );
        assert_eq!(observed["links"], 0, "mode {mode}: {observed}");
        assert_eq!(observed["results"], "", "mode {mode}: {observed}");
    }
}

/// An explicit Abort of a running target is the design's aborted/cleanup-unconfirmed terminal
/// state (browser-conformance-product.md section 3 and 7). The page must not additionally raise
/// a product `internal_failure` alert for a cancellation the user asked for.
#[test]
fn adversary_abort_of_a_running_target_is_not_reported_as_internal_failure() {
    let root = root("abort");
    let site = emit(&root);
    let (observed, channels) = observe(&root, &site, 2, true);
    assert!(
        !channels.contains(SENTINEL),
        "an issued plaintext escaped a browser channel"
    );
    assert_eq!(
        observed["during"]["state"], "Rust admission complete. Not executed.",
        "the target must still be running when Abort is pressed: {observed}"
    );
    assert_eq!(observed["during"]["abort"], false, "{observed}");
    assert!(
        observed["state"].as_str().unwrap().contains("aborted"),
        "{observed}"
    );
    assert_eq!(observed["links"], 0, "{observed}");
    assert_eq!(observed["results"], "", "{observed}");
    let alert = observed["error"].as_str().unwrap();
    assert!(
        alert.is_empty() || alert.contains("aborted"),
        "ADVERSARY_ABORT_REPORTED_AS_INTERNAL_FAILURE: the role=alert element reads `{alert}` \
         after an explicit Abort: {observed}"
    );
}
