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

/// A factory that fails with a typed product error ends as a visible execution error, with no
/// report, no download and no issued plaintext on any channel (design section 8).
#[test]
fn adversary_factory_error_ends_visibly_without_report_or_disclosure() {
    let root = root("factory-error");
    let site = emit(&root);
    let (observed, channels) = observe(&root, &site, 3, false);
    assert!(
        !channels.contains(SENTINEL),
        "an issued plaintext escaped a browser channel"
    );
    assert_eq!(observed["state"], "execution_error", "{observed}");
    assert_eq!(observed["error"], "execution_error", "{observed}");
    assert_eq!(observed["links"], 0, "{observed}");
    assert_eq!(observed["results"], "", "{observed}");
    for message in observed["messages"].as_array().unwrap() {
        assert!(
            message.get("report").is_none(),
            "a failed factory produced a report: {message}"
        );
    }
}

/// Start a run whose target never returns and poll the page until the product's own 300-second
/// watchdog ends it. Each poll is its own short `BiDi` call: one call cannot outlive the session
/// timeout, and the watchdog is the product's, not the harness's.
#[test]
fn adversary_watchdog_timeout_ends_cleanup_unconfirmed_without_report_or_disclosure() {
    const START: &str = r"(async()=>{
        const el=id=>document.getElementById(id);
        const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
        const end=performance.now()+10000;while(!window.essBrowser&&performance.now()<end)await sleep(20);
        window.adversaryMessages=[];const OriginalWorker=window.Worker;
        window.Worker=class extends OriginalWorker {constructor(...args){super(...args);this.addEventListener('message',event=>window.adversaryMessages.push(event.data));}};
        await window.essBrowser.connect();
        const loaded=el('runtime-state').textContent;
        window.adversaryStarted=performance.now();el('run').click();await sleep(1000);
        return JSON.stringify({loaded,state:el('runtime-state').textContent,abort:el('abort').disabled});
    })()";
    const POLL: &str = r"(async()=>{
        const el=id=>document.getElementById(id);
        return JSON.stringify({elapsed:performance.now()-window.adversaryStarted,
            state:el('runtime-state').textContent,error:el('error').textContent,
            results:el('results').textContent,links:document.querySelectorAll('#downloads a').length,
            runDisabled:el('run').disabled,dom:document.body.innerText,messages:window.adversaryMessages},
            (_key,value)=>value instanceof Uint8Array?new TextDecoder().decode(value):value);
    })()";
    let root = root("watchdog");
    let site = emit(&root);
    let wasm = build_host(&root, 2);
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let evidence = root.join("firefox-watchdog");
    fs::create_dir_all(&evidence).unwrap();
    let mut firefox = browser::Browser::new(&evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    firefox.subscribe_logs();
    let started = firefox.evaluate(&context, START);
    assert_eq!(
        started["state"], "Rust admission complete. Not executed.",
        "the target must still be running: {started}"
    );
    assert_eq!(started["abort"], false, "{started}");
    let begun = std::time::Instant::now();
    let observed = loop {
        std::thread::sleep(std::time::Duration::from_secs(15));
        let polled = firefox.evaluate(&context, POLL);
        if polled["state"].as_str().unwrap().contains("aborted")
            || begun.elapsed() > std::time::Duration::from_secs(400)
        {
            break polled;
        }
    };
    drop(firefox);
    let encoded = observed.to_string();
    fs::write(root.join("observed-watchdog.json"), &encoded).unwrap();
    let receipt = fs::read_to_string(evidence.join("bidi.jsonl")).unwrap();
    assert!(
        !encoded.contains(SENTINEL) && !receipt.contains(SENTINEL),
        "an issued plaintext escaped a browser channel"
    );
    assert_eq!(
        observed["state"], "aborted — cleanup unconfirmed",
        "{observed}"
    );
    let elapsed = observed["elapsed"].as_f64().unwrap();
    assert!(
        (300_000.0..360_000.0).contains(&elapsed),
        "the product watchdog, not another failure, ended the run: {elapsed} ms"
    );
    assert_eq!(observed["error"], "", "{observed}");
    assert_eq!(observed["results"], "", "{observed}");
    assert_eq!(observed["links"], 0, "{observed}");
    assert_eq!(observed["runDisabled"], true, "{observed}");
    for message in observed["messages"].as_array().unwrap() {
        assert!(message.get("report").is_none(), "{message}");
    }
}

/// After a run that issued fresh secrets completed in the worker, a Load of changed original
/// bytes is refused and the old capability is gone. Every worker message on that path, the page
/// and the `BiDi` log stay free of every issued plaintext.
#[test]
fn adversary_admission_refusal_after_an_executed_secret_discloses_nothing() {
    use ess_conformance::web_execution::{abi::load_request, bundle::hash};
    let root = root("refusal-after-execution");
    let site = emit(&root);
    let original = fs::read_to_string(site.join("browser.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&original).unwrap();
    let blobs: Vec<_> = manifest["sources"]
        .as_array()
        .unwrap()
        .iter()
        .chain([&manifest["execution"]["file"], &manifest["presentation"]])
        .map(|reference| {
            let path = reference["path"].as_str().unwrap().to_owned();
            ess_conformance::web_execution::bundle::Blob {
                bytes: fs::read(site.join(&path)).unwrap(),
                path,
            }
        })
        .collect();
    let good = load_request(1, &original, &blobs).unwrap();
    fs::write(site.join("good-load.payload"), &good[24..]).unwrap();
    let mut changed = blobs.clone();
    let execution_blob = changed.len() - 2;
    changed[execution_blob]
        .bytes
        .splice(1..1, b"\"unknown\":true,".iter().copied())
        .for_each(drop);
    let mut repinned = manifest.clone();
    repinned["execution"]["file"]["sha256"] = hash(&changed[execution_blob].bytes).into();
    repinned["execution"]["file"]["byte_length"] = changed[execution_blob].bytes.len().into();
    let mutated = load_request(3, &repinned.to_string(), &changed).unwrap();
    fs::write(site.join("mutated-load.payload"), &mutated[24..]).unwrap();
    let wasm = build_host(&root, 4);
    fs::copy(wasm, site.join("runner.wasm")).unwrap();
    let server = browser::Server::new(&site);
    let evidence = root.join("firefox-refusal");
    fs::create_dir_all(&evidence).unwrap();
    let mut firefox = browser::Browser::new(&evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    firefox.subscribe_logs();
    let observed = firefox.evaluate(
        &context,
        r"(async()=>{
        const read=async path=>new Uint8Array(await (await fetch(path)).arrayBuffer());
        const worker=new Worker('worker.js',{type:'module'}),messages=[];
        const send=(id,opcode,payload)=>new Promise(resolve=>{
            const listen=({data})=>{messages.push(data);if(data.id===id||data.kind==='fatal'){worker.removeEventListener('message',listen);resolve(data);}};
            worker.addEventListener('message',listen);
            worker.postMessage({kind:'request',id,opcode,payload},[payload.buffer]);
        });
        const run=handle=>{const bytes=new Uint8Array(24),view=new DataView(bytes.buffer);
            view.setUint32(0,handle,true);crypto.getRandomValues(bytes.subarray(4,20));view.setUint32(20,0,true);return bytes;};
        const load=await send(1,1,await read('good-load.payload'));
        const executed=await send(2,3,run(load.handle));
        const refused=await send(3,1,await read('mutated-load.payload'));
        const stale=await send(4,3,run(load.handle));
        worker.terminate();
        return JSON.stringify({handle:load.handle,loadError:load.error??null,
            executed:{error:executed.error??null,report:executed.report?new TextDecoder().decode(executed.report):null},
            refused:refused.error??null,stale:stale.error??null,messages,dom:document.body.innerText},
            (_key,value)=>value instanceof Uint8Array?new TextDecoder().decode(value):value);
    })()",
    );
    drop(firefox);
    let encoded = observed.to_string();
    fs::write(root.join("observed-refusal.json"), &encoded).unwrap();
    let receipt = fs::read_to_string(evidence.join("bidi.jsonl")).unwrap();
    assert!(
        !encoded.contains(SENTINEL) && !receipt.contains(SENTINEL),
        "an issued plaintext escaped a browser channel"
    );
    assert_ne!(observed["handle"], 0, "{observed}");
    assert_eq!(observed["loadError"], serde_json::Value::Null, "{observed}");
    assert_eq!(
        observed["executed"]["error"],
        serde_json::Value::Null,
        "{observed}"
    );
    let report: serde_json::Value =
        serde_json::from_str(observed["executed"]["report"].as_str().unwrap()).unwrap();
    assert_eq!(report["counts"]["total"], 1, "{report}");
    assert_eq!(observed["refused"], "admission_refused", "{observed}");
    assert_eq!(observed["stale"], "invalid_handle", "{observed}");
    assert_eq!(observed["messages"].as_array().unwrap().len(), 4);
}
