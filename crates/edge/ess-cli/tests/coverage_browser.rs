//! The generated generic player is exercised in actual Firefox, including admission refusal.
#[path = "support/browser.rs"]
mod browser;
#[path = "support/coverage_cases.rs"]
mod coverage_cases;
#[allow(dead_code)]
#[path = "support/executable.rs"]
mod executable;
use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn actual_browser_and_rust_refuse_every_closed_model_field_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence = TemporaryDirectory::create("ess-coverage-browser-model").unwrap();
    let generated = evidence.join("site");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .args(["--suite-format", "5", "--out"])
        .arg(&generated);
    fs::write(evidence.join("emit.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{output:?}");
    browser::legacy_replay_fixture(&generated);
    let original = fs::read_to_string(generated.join("replay.json")).unwrap();
    ess_conformance::web_replay::AdmittedReplay::from_json(&original).unwrap();
    let value: serde_json::Value = serde_json::from_str(&original).unwrap();
    let cases = model_cases(&value);
    for (name, original) in &cases {
        assert!(
            ess_conformance::web_replay::AdmittedReplay::from_json(original).is_err(),
            "Rust {name}"
        );
    }
    fs::write(
        generated.join("model-cases.json"),
        serde_json::to_string(&cases).unwrap(),
    )
    .unwrap();
    let server = browser::Server::new(&generated);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let results = browser.evaluate(
        &context,
        r"(async () => {
      const {admitReplay}=await import('./admission.js');
      const cases=await fetch('model-cases.json').then(r=>r.json());
      const results=[];
      for(const [name,original] of cases) {
        try { await admitReplay(original);results.push({name,admitted:true}); }
        catch(error) { results.push({name,admitted:false,error:String(error)}); }
      }
      return JSON.stringify(results);
    })()",
    );
    fs::write(
        evidence.join("model-results.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
    assert_eq!(results.as_array().unwrap().len(), cases.len());
    for ((name, _), result) in cases.iter().zip(results.as_array().unwrap()) {
        assert_eq!(result["name"], *name);
        assert_eq!(result["admitted"], false, "Firefox {name}: {result}");
    }
    println!("actual closed model boundary vectors: {}", cases.len());
}
fn model_cases(original: &serde_json::Value) -> Vec<(String, String)> {
    fn objects(value: &serde_json::Value, path: &str, result: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(fields) => {
                result.push(path.into());
                for (key, child) in fields {
                    objects(
                        child,
                        &format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                        result,
                    );
                }
            }
            serde_json::Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    objects(child, &format!("{path}/{index}"), result);
                }
            }
            _ => {}
        }
    }
    let mut paths = Vec::new();
    objects(&original["model"], "/model", &mut paths);
    let mut cases = Vec::new();
    for path in paths {
        let mut changed = original.clone();
        changed
            .pointer_mut(&path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("alien".into(), serde_json::json!(true));
        cases.push((format!("unknown {path}"), changed.to_string()));
        for (key, value) in original.pointer(&path).unwrap().as_object().unwrap() {
            let mut changed = original.clone();
            changed
                .pointer_mut(&path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            cases.push((format!("missing {path}/{key}"), changed.to_string()));
            let mut changed = original.clone();
            changed.pointer_mut(&path).unwrap()[key] = if value.is_array() || value.is_object() {
                serde_json::json!(true)
            } else {
                serde_json::json!([])
            };
            cases.push((format!("type {path}/{key}"), changed.to_string()));
        }
    }
    cases
}

#[test]
fn retained_legacy_player_bytes_still_replay_in_actual_firefox() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence = TemporaryDirectory::create("ess-legacy-browser").unwrap();
    let generated = evidence.join("site");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .arg("--out")
        .arg(&generated);
    fs::write(evidence.join("emit.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{output:?}");
    browser::legacy_replay_fixture(&generated);
    // Exercise the immutable historical reader explicitly; fresh products have their own vectors.
    fs::write(
        generated.join("player.js"),
        include_bytes!("../../../verify/ess-conformance/tests/fixtures/coverage/legacy-player.js"),
    )
    .unwrap();
    assert_eq!(
        fs::read(generated.join("player.js")).unwrap(),
        include_bytes!("../../../verify/ess-conformance/tests/fixtures/coverage/legacy-player.js")
    );
    let server = browser::Server::new(&generated);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(
        &context,
        r"(async () => {
      const {default:player} = await import('./player.js');
      const before = player.state.cursor;
      [...document.querySelectorAll('button')].find(b => b.textContent.startsWith('Step')).click();
      return JSON.stringify({before,after:player.state.cursor,scenarios:player.scenarios.length});
    })()",
    );
    fs::write(
        evidence.join("result.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(result["before"], -1);
    assert_eq!(result["after"], 0);
    assert_eq!(result["scenarios"], 2);
    let modern = evidence.join("modern");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .args(["--suite-format", "5", "--out"])
        .arg(&modern);
    fs::write(evidence.join("modern.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("modern.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("modern.stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{output:?}");
    browser::legacy_replay_fixture(&modern);
    let replay: serde_json::Value =
        serde_json::from_slice(&fs::read(modern.join("replay.json")).unwrap()).unwrap();
    let mut suite: serde_json::Value =
        serde_json::from_str(replay["input"]["suite_json"].as_str().unwrap()).unwrap();
    assert_eq!(suite["provenance"]["suite_version"], "ess-conformance/35");
    suite["coverage"]["knowledge"] = serde_json::json!("invented inventory claim");
    suite["provenance"]["spec_digest"] = serde_json::json!("0".repeat(64));
    fs::write(generated.join("suite.json"), suite.to_string()).unwrap();
    let context = browser.open(&format!("{}/index.html?new-metadata", server.url));
    let result = browser.evaluate(
        &context,
        r"(async () => {
      const {default:player} = await import('./player.js');
      [...document.querySelectorAll('button')].find(b => b.textContent.startsWith('Step')).click();
      return JSON.stringify({cursor:player.state.cursor,scenarios:player.scenarios.length,
        coverageBanner:document.getElementById('coverage') !== null});
    })()",
    );
    fs::write(
        evidence.join("old-player-new-metadata.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(result["cursor"], 0);
    assert_eq!(result["scenarios"], 2);
    assert_eq!(result["coverageBanner"], false);
}

#[test]
fn actual_browser_admits_the_pair_before_creating_replay_state() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence = TemporaryDirectory::create("ess-coverage-browser").unwrap();
    let generated = evidence.join("site");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .args(["--suite-format", "5", "--out"])
        .arg(&generated);
    fs::write(evidence.join("emit.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "{output:?}");
    browser::legacy_replay_fixture(&generated);
    let server = browser::Server::new(&generated);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(&context, r"(async () => {
      try {
        const {default: player} = await import('./player.js');
        const before = player.state.cursor;
        [...document.querySelectorAll('button')].find(b => b.textContent.startsWith('Step')).click();
        return JSON.stringify({admitted:true, before, after:player.state.cursor,
          selection:document.getElementById('coverage').textContent, scenarios:player.scenarios.length});
      } catch (error) { return JSON.stringify({admitted:false,error:String(error)}); }
    })()");
    fs::write(
        evidence.join("positive.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(result["admitted"], true, "{result}");
    assert_eq!(result["before"], -1);
    assert_eq!(result["after"], 0);
    assert_eq!(result["scenarios"], 2);
    assert!(result["selection"].as_str().unwrap().contains("authored"));
    let original = fs::read_to_string(generated.join("replay.json")).unwrap();
    fs::write(evidence.join("replay.original.json"), &original).unwrap();
    let mut mismatched: serde_json::Value = serde_json::from_str(&original).unwrap();
    mismatched["model"]["contract_digest"] = serde_json::json!("0".repeat(64));
    fs::write(
        evidence.join("replay.mismatched.json"),
        mismatched.to_string(),
    )
    .unwrap();
    fs::write(generated.join("replay.json"), mismatched.to_string()).unwrap();
    let refused_context = browser.open(&format!("{}/index.html?mismatched", server.url));
    let refused = browser.evaluate(
        &refused_context,
        r"(async () => {
      try { await import('./player.js'); return JSON.stringify({admitted:true}); }
      catch(error) { return JSON.stringify({admitted:false,error:String(error),
        banner:document.getElementById('coverage') !== null,
        mounted:document.getElementById('app').hasAttribute('data-v-app')}); }
    })()",
    );
    fs::write(
        evidence.join("mismatched.json"),
        serde_json::to_string_pretty(&refused).unwrap(),
    )
    .unwrap();
    assert_eq!(refused["admitted"], false, "{refused}");
    assert_eq!(refused["banner"], false);
    assert_eq!(refused["mounted"], false);
    let mut invalid: serde_json::Value = serde_json::from_str(&original).unwrap();
    invalid["model"]["entities"][0]["display"] = serde_json::json!("INVALID-BYTE");
    let mut bytes = invalid.to_string().into_bytes();
    let at = bytes
        .windows(12)
        .position(|window| window == b"INVALID-BYTE")
        .unwrap();
    bytes[at] = 0xff;
    fs::write(generated.join("replay.json"), &bytes).unwrap();
    fs::write(evidence.join("replay.invalid-utf8.json"), &bytes).unwrap();
    let invalid_context = browser.open(&format!("{}/index.html?invalid-utf8", server.url));
    let invalid_result = browser.evaluate(
        &invalid_context,
        r"(async () => {
      try { await import('./player.js'); return JSON.stringify({admitted:true}); }
      catch(error) { return JSON.stringify({admitted:false,error:String(error),
        banner:document.getElementById('coverage') !== null,
        mounted:document.getElementById('app').hasAttribute('data-v-app')}); }
    })()",
    );
    fs::write(
        evidence.join("invalid-utf8.json"),
        serde_json::to_string_pretty(&invalid_result).unwrap(),
    )
    .unwrap();
    assert_eq!(invalid_result["admitted"], false, "{invalid_result}");
    assert_eq!(invalid_result["banner"], false);
    assert_eq!(invalid_result["mounted"], false);
}

#[test]
fn actual_browser_checks_full_lineage_and_integer_metadata() {
    let evidence = TemporaryDirectory::create("ess-coverage-browser-lineage").unwrap();
    fs::write(
        evidence.join("index.html"),
        "<!doctype html><title>Coverage admission vectors</title>",
    )
    .unwrap();
    fs::write(
        evidence.join("admission.js"),
        include_str!("../../../verify/ess-conformance/assets/coverage-admission.js"),
    )
    .unwrap();
    let cases = coverage_cases::cases();
    for case in &cases {
        let result = ess_conformance::coverage::AdmittedInput::from_json(&case.input);
        assert_eq!(
            result.is_ok(),
            case.accepted,
            "Rust {}: {result:?}",
            case.name
        );
    }
    fs::write(
        evidence.join("cases.json"),
        serde_json::to_string_pretty(&cases).unwrap(),
    )
    .unwrap();
    let server = browser::Server::new(&evidence);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(&context, r#"(async () => {
      const {admitReplay} = await import('./admission.js');
      const cases = await fetch('cases.json').then(r => r.json());
      const model = {system:'example',version:'v1',spec_digest:'a'.repeat(64),contract_digest:'b'.repeat(64),
        entities:[],commands:[],views:[],actors:[],bindings:[]};
      const results = [];
      for (const test of cases) {
        try {
          const input = JSON.parse(test.input);
          const digest = 'sha256:' + [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(input.suite_json)))].map(n => n.toString(16).padStart(2,'0')).join('');
          const suite = {version:JSON.parse(input.suite_json).provenance.suite_version,digest_profile:'sha256-json-bytes/1',digest};
          const original = '{' + ['\"format\":\"ess-conformance-replay/1\"','\"model\":'+JSON.stringify(model),'\"suite\":'+JSON.stringify(suite),'\"input\":'+test.input].join(',') + '}';
          const admitted = await admitReplay(original);
          results.push({name:test.name,admitted:true,description:admitted.description});
        } catch(error) { results.push({name:test.name,admitted:false,error:String(error)}); }
      }
      return JSON.stringify(results);
    })()"#);
    fs::write(
        evidence.join("results.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    let observed = result.as_array().unwrap();
    assert_eq!(observed.len(), cases.len());
    for (case, result) in cases.iter().zip(observed) {
        assert_eq!(result["name"], case.name);
        assert_eq!(
            result["admitted"], case.accepted,
            "Firefox {}: {result}",
            case.name
        );
        if case.name == "repeated-in-scope-refusals" {
            let description = result["description"].as_str().unwrap();
            assert_eq!(
                description.matches("ESS-AUTHOR-001").count(),
                2,
                "{description}"
            );
            assert_eq!(
                description.matches("An original refused candidate").count(),
                2,
                "{description}"
            );
            assert!(description.contains("incomplete"), "{description}");
        }
    }
}

// The three tests above start Firefox at the same moment. On a shared runner
// they then miss one shared startup deadline together and read as a BiDi
// protocol defect. These three cases decide how a start the fixture did not
// get is reported, and that the fixture never asks for three starts at once.
#[test]
fn a_start_past_the_deadline_is_a_fixture_environment_refusal_not_a_bidi_defect() {
    let evidence = TemporaryDirectory::create("ess-browser-startup-deadline").unwrap();
    // No Firefox on any runner reaches BiDi readiness before an elapsed deadline.
    let Err(refusal) = browser::Browser::launch(&evidence, Duration::ZERO) else {
        panic!("an elapsed deadline admitted a browser")
    };
    fs::write(evidence.join("startup-refusal.txt"), &refusal).unwrap();
    assert!(
        refusal.starts_with("fixture environment refusal:"),
        "{refusal}"
    );
    assert!(refusal.contains("not a BiDi protocol defect"), "{refusal}");
    assert!(refusal.contains("stage:"), "{refusal}");
    assert!(refusal.contains("measured startup: 0."), "{refusal}");
    assert!(
        refusal.contains("deadline:         0.000s (expired)"),
        "{refusal}"
    );
    assert!(
        refusal.contains(&evidence.display().to_string()),
        "{refusal}"
    );
    let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
    assert!(
        refusal.contains(&format!("firefox.stderr ({} bytes):", log.len())),
        "{refusal}"
    );
}

/// A slow runner is not a defect in the page under test, so the fixture's own waits are sized for
/// the slowest runner seen rather than for a quiet one. `main` a1cf7233f lost a real start at
/// 30.010s of a 30s budget (job 108296632588, stage `announce`) and a `BiDi` read at the old 20s
/// session timeout earlier the same day; both budgets are two minutes now, and this case is what
/// fails if a later speed pass trims them back into the range a loaded runner reaches.
#[test]
fn the_fixture_waits_minutes_for_a_loaded_runner_before_giving_up() {
    let loaded_runner = Duration::from_secs(120);
    assert!(
        browser::STARTUP_DEADLINE >= loaded_runner,
        "a Firefox start is given {:?}; a loaded runner has taken past 30s",
        browser::STARTUP_DEADLINE
    );
    assert!(
        browser::SESSION_TIMEOUT >= loaded_runner,
        "a BiDi call is given {:?}; a loaded runner has taken past 20s",
        browser::SESSION_TIMEOUT
    );
}

/// The variant names of one enum, read out of the fixture's own source. A case
/// that walks a list of stages has to walk the stages the fixture declares; a
/// literal list is a claim about the fixture that nothing checks, and the last
/// one was already wrong by a stage on the day it was written.
fn declared_variants(source: &str, declaration: &str) -> Vec<String> {
    let body = source
        .split_once(declaration)
        .expect("the fixture declares it")
        .1
        .split_once('{')
        .expect("the declaration has a body")
        .1;
    let body = &body[..body.find("\n}").expect("the body ends at column zero")];
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .map(|line| {
            line.split(['(', ',', '{'])
                .next()
                .expect("a variant name")
                .trim()
                .to_owned()
        })
        .collect()
}

#[test]
fn a_startup_refusal_attaches_the_stderr_firefox_actually_wrote() {
    let evidence = TemporaryDirectory::create("ess-browser-startup-evidence").unwrap();
    let written = "*** You are running in headless mode.\nthis runner was busy\n";
    fs::write(evidence.join("firefox.stderr"), written).unwrap();
    // Every way the fixture gives up on a start reports the same way, and the
    // list of ways is the fixture's own `Stage`, held here against the variants
    // its source declares rather than against three names typed into this case.
    let declared = declared_variants(include_str!("support/browser.rs"), "pub enum Stage");
    assert_eq!(
        browser::Stage::ALL.len(),
        declared.len(),
        "Stage::ALL names {} stages and the fixture declares {}: {declared:?}",
        browser::Stage::ALL.len(),
        declared.len()
    );
    for (stage, variant) in browser::Stage::ALL.iter().zip(&declared) {
        let label = stage.label();
        assert!(
            label.starts_with(&variant.to_lowercase()),
            "Stage::ALL is out of step with the declared variants: {label} against {variant}"
        );
        // A 30s budget given explicitly: this case holds how a refusal prints a measurement
        // beside a deadline, not what the fixture's own budget is.
        let refusal = browser::startup_refusal(
            *stage,
            Duration::from_millis(31_500),
            Duration::from_secs(30),
            &evidence,
        );
        assert!(
            refusal.starts_with("fixture environment refusal:"),
            "{refusal}"
        );
        assert!(refusal.contains("not a BiDi protocol defect"), "{refusal}");
        assert!(
            refusal.contains(&format!("stage:            {label}")),
            "{refusal}"
        );
        // A stage that never timed a startup says so instead of printing a
        // number beside a deadline that number was never compared against.
        if matches!(stage, browser::Stage::Spawn(_)) {
            assert!(
                refusal.contains("measured startup: not timed; the browser was never spawned"),
                "{refusal}"
            );
            assert!(!refusal.contains("deadline:"), "{refusal}");
        } else {
            assert!(
                refusal.contains("measured startup: 31.500s (spawn to give-up)"),
                "{refusal}"
            );
            assert!(
                refusal.contains("deadline:         30.000s (expired)"),
                "{refusal}"
            );
        }
        assert!(
            refusal.contains(&format!(
                "firefox.stderr ({} bytes):\n{written}",
                written.len()
            )),
            "{refusal}"
        );
    }
}

/// The panic forms the startup guard reads on every line a start runs. A panic raised inside
/// another file or crate, an index or slice out of bounds and an arithmetic overflow are not
/// among them.
const PANIC_FORMS: &[&str] = &[
    ".unwrap()",
    ".unwrap_err()",
    ".expect(",
    ".expect_err(",
    "panic!",
    "assert!",
    "assert_eq!",
    "assert_ne!",
    "unreachable!",
    "todo!",
    "unimplemented!",
];

/// The two exceptions a line that can end a start may name, read from the comment on the line
/// itself or on the line above it: `harness`, this runner's own filesystem or process table, and
/// `defect`, a malformed upgrade or protocol answer.
const STARTUP_MARKS: &[&str] = &["// startup-path: harness", "// startup-path: defect"];

/// Where a start begins: the functions a caller asks for a browser through. `Browser::new` is not
/// one, because its panic is how a caller that takes no `Result` receives the refusal itself.
const STARTS: &[(&str, &str)] = &[("Browser", "launch"), ("Browser", "launch_program")];

/// One source file with every comment and literal blanked out of its code, line for line, and the
/// text of each line's comments kept apart. Nothing a string, a character literal or a comment
/// holds can open, close or call anything in the code that is left.
struct Blanker {
    chars: Vec<char>,
    at: usize,
    code: String,
    comments: Vec<String>,
}

impl Blanker {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at + ahead).copied()
    }
    fn keep(&mut self) {
        let Some(c) = self.peek(0) else { return };
        self.at += 1;
        if c == '\n' {
            self.comments.push(String::new());
        }
        self.code.push(if c.is_ascii() { c } else { ' ' });
    }
    fn hide(&mut self, comment: bool, count: usize) {
        for _ in 0..count {
            let Some(c) = self.peek(0) else { return };
            self.at += 1;
            if c == '\n' {
                self.comments.push(String::new());
                self.code.push('\n');
                continue;
            }
            if comment {
                self.comments.last_mut().expect("a line").push(c);
            }
            self.code.push(' ');
        }
    }
    fn string(&mut self) {
        self.hide(false, 1);
        while let Some(c) = self.peek(0) {
            match c {
                '\\' => self.hide(false, 2),
                '"' => {
                    self.hide(false, 1);
                    return;
                }
                _ => self.hide(false, 1),
            }
        }
    }
    fn raw_string(&mut self, hashes: usize) {
        self.hide(false, hashes + 1);
        while let Some(c) = self.peek(0) {
            if c == '"' && (1..=hashes).all(|ahead| self.peek(ahead) == Some('#')) {
                self.hide(false, hashes + 1);
                return;
            }
            self.hide(false, 1);
        }
    }
    fn quote(&mut self) {
        if self.peek(1) == Some('\\') {
            let close = (self.at + 3..self.chars.len()).find(|&index| self.chars[index] == '\'');
            let end = close.map_or(self.chars.len(), |index| index + 1);
            self.hide(false, end - self.at);
        } else if self.peek(2) == Some('\'') {
            self.hide(false, 3);
        } else {
            // A lifetime or a label.
            self.keep();
        }
    }
    fn run(mut self) -> (String, Vec<String>) {
        while let Some(c) = self.peek(0) {
            let next = self.peek(1);
            if c == '/' && next == Some('/') {
                while self.peek(0).is_some_and(|c| c != '\n') {
                    self.hide(true, 1);
                }
            } else if c == '/' && next == Some('*') {
                let mut depth = 0_usize;
                while self.peek(0).is_some() {
                    if self.peek(0) == Some('/') && self.peek(1) == Some('*') {
                        depth += 1;
                        self.hide(true, 2);
                    } else if self.peek(0) == Some('*') && self.peek(1) == Some('/') {
                        depth -= 1;
                        self.hide(true, 2);
                        if depth == 0 {
                            break;
                        }
                    } else {
                        self.hide(true, 1);
                    }
                }
            } else if c == '"' {
                self.string();
            } else if c == '\'' {
                self.quote();
            } else if c.is_alphanumeric() || c == '_' {
                let end = (self.at..self.chars.len())
                    .find(|&index| {
                        !(self.chars[index].is_alphanumeric() || self.chars[index] == '_')
                    })
                    .unwrap_or(self.chars.len());
                let word: String = self.chars[self.at..end].iter().collect();
                let hashes = self.chars[end..].iter().take_while(|c| **c == '#').count();
                let after = self.chars.get(end).copied();
                if matches!(word.as_str(), "r" | "br") && self.chars.get(end + hashes) == Some(&'"')
                {
                    self.hide(false, word.len());
                    self.raw_string(hashes);
                } else if word == "b" && after == Some('"') {
                    self.hide(false, 1);
                    self.string();
                } else if word == "b" && after == Some('\'') {
                    self.hide(false, 1);
                    self.quote();
                } else {
                    for _ in self.at..end {
                        self.keep();
                    }
                }
            } else {
                self.keep();
            }
        }
        (self.code, self.comments)
    }
}

/// A token of blanked code and the 1-based line it is on.
struct Token<'a> {
    text: &'a str,
    line: usize,
}

fn tokens(code: &str) -> Vec<Token<'_>> {
    let bytes = code.as_bytes();
    let mut found = Vec::new();
    let mut line = 1;
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte == b'\n' {
            line += 1;
            at += 1;
        } else if byte.is_ascii_whitespace() {
            at += 1;
        } else if byte.is_ascii_alphanumeric() || byte == b'_' {
            let end = (at..bytes.len())
                .find(|&index| !(bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_'))
                .unwrap_or(bytes.len());
            found.push(Token {
                text: &code[at..end],
                line,
            });
            at = end;
        } else {
            found.push(Token {
                text: &code[at..=at],
                line,
            });
            at += 1;
        }
    }
    found
}

/// The token that closes the bracket opened at `open`.
fn closing(tokens: &[Token<'_>], open: usize) -> Option<usize> {
    let (opens, closes) = match tokens[open].text {
        "{" => ("{", "}"),
        "(" => ("(", ")"),
        "<" => ("<", ">"),
        _ => return None,
    };
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        if token.text == opens {
            depth += 1;
        } else if token.text == closes {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn is_word(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
}

/// One function the source declares with a body.
struct Function {
    owner: Option<String>,
    name: String,
    method: bool,
    body: (usize, usize),
    lines: (usize, usize),
}

impl Function {
    fn path(&self) -> String {
        self.owner.as_ref().map_or_else(
            || self.name.clone(),
            |owner| format!("{owner}::{}", self.name),
        )
    }
}

/// What a start can reach in one source file.
struct StartupScan {
    /// `Owner::name` (or a free function's name) of every function a start can run, with the
    /// first and last line read for it.
    reached: Vec<(String, usize, usize)>,
    /// A line a start runs that holds a panic form and names no exception.
    unaccounted: Vec<(usize, String)>,
    /// A line a start runs that holds a panic form and names its exception.
    accounted: Vec<(usize, String)>,
    /// A mark that names no panic site a start runs, on its own line or the next.
    stale: Vec<(usize, String)>,
}

impl StartupScan {
    fn reaches(&self, path: &str) -> bool {
        self.reached.iter().any(|(reached, _, _)| reached == path)
    }
}

/// Follow every same-file call a start can make, and read every line of every function it reaches.
#[allow(
    clippy::too_many_lines,
    reason = "one pass from items to reach to sites keeps the scan's rules in one reading"
)]
fn startup_scan(source: &str) -> Result<StartupScan, String> {
    let (code, comments) = Blanker {
        chars: source.chars().collect(),
        at: 0,
        code: String::new(),
        comments: vec![String::new()],
    }
    .run();
    let tokens = tokens(&code);
    let text = |index: usize| tokens.get(index).map_or("", |token| token.text);
    let line = |index: usize| tokens.get(index).map_or(0, |token| token.line);

    // `impl` blocks, as (owner type, trait, first token, last token).
    let mut impls: Vec<(String, Option<String>, usize, usize)> = Vec::new();
    for index in 0..tokens.len() {
        let item = index == 0 || matches!(text(index - 1), "}" | ";" | "]" | "{");
        if text(index) != "impl" || !item {
            continue;
        }
        let mut at = index + 1;
        if text(at) == "<" {
            at = closing(&tokens, at).ok_or("an impl's generics never close")? + 1;
        }
        let header_start = at;
        while at < tokens.len() && text(at) != "{" {
            at += 1;
        }
        let header: Vec<&str> = (header_start..at).map(text).collect();
        let first_word = |words: &[&str]| {
            words
                .iter()
                .find(|word| is_word(word))
                .map(|word| (*word).to_owned())
        };
        let (owner, implemented) = match header.iter().position(|word| *word == "for") {
            Some(split) => (
                first_word(&header[split + 1..]),
                first_word(&header[..split]),
            ),
            None => (first_word(&header), None),
        };
        let owner = owner.ok_or("an impl names no type")?;
        let end = closing(&tokens, at).ok_or("an impl block never closes")?;
        impls.push((owner, implemented, at, end));
    }

    let mut functions: Vec<Function> = Vec::new();
    for index in 0..tokens.len() {
        if text(index) != "fn" || !is_word(text(index + 1)) {
            continue;
        }
        let Some(open) = (index + 2..tokens.len()).find(|&at| text(at) == "(") else {
            continue;
        };
        let close = closing(&tokens, open).ok_or("a parameter list never closes")?;
        let method = (open..close).any(|at| text(at) == "self");
        let Some(body) = (close..tokens.len()).find(|&at| matches!(text(at), "{" | ";")) else {
            continue;
        };
        if text(body) == ";" {
            continue;
        }
        let end = closing(&tokens, body).ok_or("a function body never closes")?;
        let owner = impls
            .iter()
            .filter(|(_, _, first, last)| *first < index && index < *last)
            .min_by_key(|(_, _, first, last)| last - first)
            .map(|(owner, _, _, _)| owner.clone());
        functions.push(Function {
            owner,
            name: text(index + 1).to_owned(),
            method,
            body: (body, end),
            lines: (line(index), line(end)),
        });
    }

    // Every `struct` and `enum`, with the tokens that declare what it holds.
    let mut types: Vec<(String, usize, usize)> = Vec::new();
    for index in 0..tokens.len() {
        if !matches!(text(index), "struct" | "enum") || !is_word(text(index + 1)) {
            continue;
        }
        let mut at = index + 2;
        if text(at) == "<" {
            at = closing(&tokens, at).ok_or("a type's generics never close")? + 1;
        }
        let end = if matches!(text(at), "{" | "(") {
            closing(&tokens, at).ok_or("a type's body never closes")?
        } else {
            at
        };
        types.push((text(index + 1).to_owned(), at, end));
    }
    let dropped: Vec<&str> = impls
        .iter()
        .filter(|(_, implemented, _, _)| implemented.as_deref() == Some("Drop"))
        .map(|(owner, _, _, _)| owner.as_str())
        .collect();
    let is_type =
        |word: &str| dropped.contains(&word) || types.iter().any(|(name, _, _)| name == word);

    let mut reached = vec![false; functions.len()];
    let mut named: Vec<String> = Vec::new();
    let mut pending: Vec<usize> = Vec::new();
    for (owner, name) in STARTS {
        let start = functions
            .iter()
            .position(|function| function.owner.as_deref() == Some(owner) && function.name == *name)
            .ok_or_else(|| format!("the source declares no `{owner}::{name}` to start from"))?;
        reached[start] = true;
        pending.push(start);
    }
    // A type is named by the start: its `Drop` runs, and so do the `Drop`s of what it holds.
    let name_type =
        |word: &str, named: &mut Vec<String>, pending: &mut Vec<usize>, reached: &mut [bool]| {
            let mut types_pending = vec![word.to_owned()];
            while let Some(word) = types_pending.pop() {
                if named.contains(&word) || !is_type(&word) {
                    continue;
                }
                named.push(word.clone());
                for (index, function) in functions.iter().enumerate() {
                    if function.owner.as_deref() == Some(word.as_str())
                        && function.name == "drop"
                        && dropped.contains(&word.as_str())
                        && !reached[index]
                    {
                        reached[index] = true;
                        pending.push(index);
                    }
                }
                for (name, first, last) in &types {
                    if *name == word {
                        types_pending.extend(
                            (*first..=*last)
                                .map(text)
                                .filter(|held| is_type(held))
                                .map(str::to_owned),
                        );
                    }
                }
            }
        };
    while let Some(current) = pending.pop() {
        let owner = functions[current].owner.clone();
        let (first, last) = functions[current].body;
        let mut calls: Vec<usize> = Vec::new();
        for at in first..=last {
            let word = text(at);
            if !is_word(word) {
                if word == "."
                    && is_word(text(at + 1))
                    && (text(at + 2) == "(" || text(at + 2) == ":")
                {
                    let method = text(at + 1);
                    calls.extend(
                        functions
                            .iter()
                            .enumerate()
                            .filter(|(_, function)| function.method && function.name == method)
                            .map(|(index, _)| index),
                    );
                }
                continue;
            }
            let word = if word == "Self" {
                owner.as_deref().unwrap_or(word)
            } else {
                word
            };
            name_type(word, &mut named, &mut pending, &mut reached);
            if text(at + 1) == ":" && text(at + 2) == ":" && is_word(text(at + 3)) {
                let name = text(at + 3);
                calls.extend(
                    functions
                        .iter()
                        .enumerate()
                        .filter(|(_, function)| {
                            function.owner.as_deref() == Some(word) && function.name == name
                        })
                        .map(|(index, _)| index),
                );
            }
            let in_path = text(at.wrapping_sub(1)) == ":" || text(at.wrapping_sub(1)) == ".";
            if !in_path && text(at.wrapping_sub(1)) != "fn" {
                calls.extend(
                    functions
                        .iter()
                        .enumerate()
                        .filter(|(_, function)| function.owner.is_none() && function.name == word)
                        .map(|(index, _)| index),
                );
            }
        }
        for index in calls {
            if !reached[index] {
                reached[index] = true;
                pending.push(index);
            }
        }
    }

    let lines: Vec<&str> = source.lines().collect();
    let code_lines: Vec<&str> = code.split('\n').collect();
    let marked = |number: usize| {
        comments.get(number.wrapping_sub(1)).is_some_and(|comment| {
            STARTUP_MARKS
                .iter()
                .any(|mark| comment.trim_end().ends_with(mark))
        })
    };
    let raw = |number: usize| {
        lines
            .get(number.wrapping_sub(1))
            .map_or("", |line| *line)
            .to_owned()
    };
    let mut run: Vec<usize> = functions
        .iter()
        .zip(&reached)
        .filter(|(_, reached)| **reached)
        .flat_map(|(function, _)| function.lines.0..=function.lines.1)
        .collect();
    run.sort_unstable();
    run.dedup();
    let site = |number: usize| {
        code_lines
            .get(number.wrapping_sub(1))
            .is_some_and(|line| PANIC_FORMS.iter().any(|form| line.contains(form)))
    };
    let mut scan = StartupScan {
        reached: functions
            .iter()
            .zip(&reached)
            .filter(|(_, reached)| **reached)
            .map(|(function, _)| (function.path(), function.lines.0, function.lines.1))
            .collect(),
        unaccounted: Vec::new(),
        accounted: Vec::new(),
        stale: Vec::new(),
    };
    for &number in &run {
        if site(number) {
            if marked(number) || marked(number - 1) {
                scan.accounted.push((number, raw(number)));
            } else {
                scan.unaccounted.push((number, raw(number)));
            }
        }
    }
    for number in 1..=comments.len() {
        let names_a_site = [number, number + 1]
            .iter()
            .any(|line| run.binary_search(line).is_ok() && site(*line));
        if marked(number) && !names_a_site {
            scan.stale.push((number, raw(number)));
        }
    }
    Ok(scan)
}

/// The class F1 named: a start this process loses reports through `startup_refusal`, and the only
/// lines a start runs that may end it any other way are the ones that say which exception they
/// are. The sixth give-up site was found by an adversary because the enumeration behind that claim
/// was a reading of the file, and the marked region that replaced the reading saw neither a panic
/// one call below a line in it nor a forged end that made it shorter
/// (`story:a-marked-region-is-not-a-scan-of-what-runs`).
///
/// So this reads the compiler's own copy of `support/browser.rs` and follows what a start runs:
/// from `Browser::launch` and `Browser::launch_program` (`STARTS`), every same-file function a
/// reached body calls by `Self::f` or `Type::f`, every same-file method named in a `.f(` call (all
/// of them, since the scan does not know the receiver's type), every same-file free function a
/// body names, and the `Drop` of every same-file type a reached body names, directly or through
/// what that type holds. Comments, strings and character literals are blanked first, so no text in
/// them opens, closes or calls anything. Every line of every reached function, from its `fn` line
/// to its closing brace, is read for `PANIC_FORMS`; a line holding one names `startup-path:
/// harness` or `startup-path: defect` in a comment on that line or the one above, and a mark that
/// names no such line is stale. A panic inside another file or crate, an index, an arithmetic
/// overflow, and a call through a trait object or a function pointer the file never names are
/// outside what it reads. The `the_startup_scan_*` cases hold the scan itself to synthetic sources.
#[test]
fn no_unaccounted_panic_site_can_end_a_start() {
    use std::fmt::Write as _;
    let scan =
        startup_scan(include_str!("support/browser.rs")).unwrap_or_else(|error| panic!("{error}"));
    let listed = |sites: &[(usize, String)]| {
        sites
            .iter()
            .fold(String::new(), |mut text, (number, line)| {
                let _ = writeln!(text, "  support/browser.rs:{number}: {}", line.trim());
                text
            })
    };
    assert!(
        scan.unaccounted.is_empty(),
        "{} line(s) a start runs can end it without a fixture environment refusal and name no \
         reason. Give up through startup_refusal, or mark the line `startup-path: harness` (this \
         runner's own filesystem or process table) or `startup-path: defect` (a malformed upgrade \
         or protocol answer):\n{}",
        scan.unaccounted.len(),
        listed(&scan.unaccounted)
    );
    assert!(
        scan.stale.is_empty(),
        "{} startup mark(s) name no panic site a start runs, on their own line or the next:\n{}",
        scan.stale.len(),
        listed(&scan.stale)
    );
    // What a start is known to run. A scan that stops following calls drops one of these, which
    // is what a shorter region looked like.
    for path in [
        "Browser::launch_program",
        "Browser::reach_bidi",
        "Browser::upgrade",
        "Browser::connect_give_up",
        "Browser::assigned_bidi_port",
        "Browser::exchange",
        "Browser::send",
        "Browser::receive",
        "startup_refusal",
        "Stage::label",
        "Stage::timed_a_startup",
        "ProfileScratch::create",
        "ProfileScratch::drop",
        "OwnedChild::stop",
        "OwnedChild::drop",
        "Starting::begin",
        "InStartup::begin",
        "InStartup::drop",
    ] {
        assert!(
            scan.reaches(path),
            "a start runs `{path}` and the scan did not reach it: {:?}",
            scan.reached
        );
    }
    assert!(
        !scan.accounted.is_empty(),
        "the scan read no marked site at all"
    );
}

/// A synthetic fixture for the scan itself. Each `// AT_<NAME>` line is where a case puts a site:
/// in a start, one call down by method, two calls down in a free function, in the `Drop` of a type
/// the start builds, and in two places no start runs. Before `AT_START`, a string, a raw string
/// line at the function's own indentation, a character literal, a block comment and the old end
/// marker each hold a closing brace or an end, and none of them ends the function.
const SYNTHETIC: &str = r#"
pub struct Browser {
    guard: Guard,
}
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        // AT_DROP
    }
}
struct Unbuilt;
impl Drop for Unbuilt {
    fn drop(&mut self) {
        // AT_UNBUILT_DROP
    }
}
impl Browser {
    pub fn new() -> Self {
        Self::launch().unwrap_or_else(|refusal| panic!("{refusal}"))
    }
    pub fn launch() -> Result<Self, String> {
        Self::launch_program()
    }
    pub fn launch_program() -> Result<Self, String> {
        let mut browser = Self { guard: Guard };
        let text = "}";
        let raw = r"
    }
";
        let brace = '}';
        /* } */
        // startup-path: end
        browser.handshake(text, raw, brace);
        // AT_START
        Ok(browser)
    }
    fn handshake(&mut self, text: &str, raw: &str, brace: char) {
        // AT_METHOD
        refuse(text, raw, brace);
    }
    fn unreached(&self) {
        // AT_UNREACHED
    }
}
fn refuse(text: &str, raw: &str, brace: char) {
    // AT_FREE
    let _ = (text, raw, brace);
}
"#;

/// `SYNTHETIC` with one placeholder replaced, and the line the replacement is on.
fn synthetic_with(at: &str, line: &str) -> (String, usize) {
    let source = SYNTHETIC.replace(&format!("// {at}\n"), &format!("{line}\n"));
    let number = source
        .lines()
        .position(|text| text.contains(line.lines().next().unwrap_or(line).trim()))
        .unwrap_or_else(|| panic!("{at} is in the synthetic fixture"))
        + 1;
    (source, number)
}

#[test]
fn the_startup_scan_sees_a_panic_one_and_two_calls_below_the_start_and_in_a_drop_it_runs() {
    for at in ["AT_START", "AT_METHOD", "AT_FREE", "AT_DROP"] {
        let (source, number) = synthetic_with(at, &format!("        Some(\"{at}\").unwrap();"));
        let scan = startup_scan(&source).unwrap();
        assert!(
            scan.unaccounted.iter().any(|(line, _)| *line == number),
            "a panic at {at} (line {number}) is run by a start and the scan did not report it: \
             reached {:?}, unaccounted {:?}",
            scan.reached,
            scan.unaccounted
        );
    }
}

#[test]
fn the_startup_scan_reads_a_start_to_its_closing_brace_past_text_that_looks_like_an_end() {
    let (source, number) = synthetic_with("AT_START", "        Some(\"after\").unwrap();");
    let scan = startup_scan(&source).unwrap();
    let close = source
        .lines()
        .position(|line| line == "        Ok(browser)")
        .expect("the start returns")
        + 2;
    assert!(
        scan.reached
            .iter()
            .any(|(path, _, last)| path == "Browser::launch_program" && *last == close),
        "the start ends at line {close}: {:?}",
        scan.reached
    );
    assert_eq!(
        scan.unaccounted
            .iter()
            .map(|(line, _)| *line)
            .collect::<Vec<_>>(),
        [number],
        "a site after a forged end, a brace in a string, a raw string and a character literal"
    );
}

#[test]
fn the_startup_scan_leaves_out_what_no_start_runs() {
    for at in ["AT_UNREACHED", "AT_UNBUILT_DROP"] {
        let (source, number) = synthetic_with(at, &format!("        Some(\"{at}\").unwrap();"));
        let scan = startup_scan(&source).unwrap();
        assert!(
            scan.unaccounted.is_empty(),
            "{at} (line {number}) is run by no start: {:?}",
            scan.unaccounted
        );
    }
    let scan = startup_scan(SYNTHETIC).unwrap();
    assert!(!scan.reaches("Browser::new"), "{:?}", scan.reached);
    assert!(!scan.reaches("Unbuilt::drop"), "{:?}", scan.reached);
    assert!(scan.reaches("refuse"), "{:?}", scan.reached);
    let missing = SYNTHETIC.replace("fn launch_program(", "fn launch_elsewhere(");
    let Err(refusal) = startup_scan(&missing) else {
        panic!("a source without the start was scanned")
    };
    assert!(refusal.contains("Browser::launch_program"), "{refusal}");
}

#[test]
fn a_startup_mark_is_read_from_a_comment_and_names_a_site_a_start_runs() {
    let marked = "        // startup-path: harness\n        Some(1).unwrap();";
    let (source, number) = synthetic_with("AT_METHOD", marked);
    let scan = startup_scan(&source).unwrap();
    assert!(scan.unaccounted.is_empty(), "{:?}", scan.unaccounted);
    assert_eq!(
        scan.accounted
            .iter()
            .map(|(line, _)| *line)
            .collect::<Vec<_>>(),
        [number + 1]
    );
    // A mark that a string holds is not a comment, and accounts for nothing.
    let quoted = "        let _ = \"\n// startup-path: harness\n\";\n        Some(2).unwrap();";
    let (source, _) = synthetic_with("AT_METHOD", quoted);
    let scan = startup_scan(&source).unwrap();
    assert_eq!(scan.unaccounted.len(), 1, "{:?}", scan.unaccounted);
    // A mark beside no site, or on a line no start runs, is stale.
    for at in ["AT_METHOD", "AT_UNREACHED"] {
        let (source, number) = synthetic_with(at, "        // startup-path: defect");
        let scan = startup_scan(&source).unwrap();
        assert_eq!(
            scan.stale.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            [number],
            "{at}"
        );
    }
}

#[test]
fn a_browser_that_exits_during_startup_refuses_with_its_own_stderr_too() {
    let evidence = TemporaryDirectory::create("ess-browser-startup-exit").unwrap();
    // A program that is not Firefox exits at once and writes its own reason. The 30s budget is
    // explicit because this case holds how an unexpired deadline is printed, not its size.
    let Err(refusal) = browser::Browser::launch_program(
        &evidence,
        env!("CARGO_BIN_EXE_ess").as_ref(),
        Duration::from_secs(30),
    ) else {
        panic!("a browser that exited admitted a session")
    };
    fs::write(evidence.join("exit-refusal.txt"), &refusal).unwrap();
    assert!(
        refusal.starts_with("fixture environment refusal:"),
        "{refusal}"
    );
    assert!(refusal.contains("not a BiDi protocol defect"), "{refusal}");
    assert!(refusal.contains("stage:            exited"), "{refusal}");
    // The measurement is the time from spawn to the exit, and it is not a
    // startup this runner was too slow for: pinning it at "0." would pin noise,
    // so what this case holds is that the report says which one it is.
    assert!(
        refusal.contains("measured startup: 0.") && refusal.contains("(spawn to give-up)"),
        "{refusal}"
    );
    assert!(
        refusal.contains("deadline:         30.000s (not reached"),
        "{refusal}"
    );
    let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
    assert!(!log.is_empty(), "the child wrote nothing to stderr");
    assert!(
        refusal.contains(&format!("firefox.stderr ({} bytes):\n{log}", log.len())),
        "{refusal}"
    );
}

/// A stand-in script this binary installs runs at once, even while other cases fork children.
///
/// Every browser case here installs a stand-in script and executes it, as threads of one process.
/// A child another thread forks while the installer holds the script open for writing inherits
/// that descriptor until it executes, and until then the kernel refuses to execute the script
/// (`ETXTBSY`), so the startup under test would fail at launch for a reason that is not the
/// fixture's. This installs forty and runs each once while three threads spawn.
#[test]
fn a_freshly_installed_stand_in_script_runs_while_other_cases_spawn() {
    let root = TemporaryDirectory::create("ess-browser-installed").unwrap();
    let unlaunched = executable::unlaunched_rounds(40, |round| {
        let script = root.join(format!("stand-in-{round}.sh"));
        executable::install_bytes(&script, b"#!/bin/sh\nexit 0\n").unwrap();
        Command::new(script)
    });
    let _ = fs::remove_dir_all(&root);
    assert!(
        unlaunched.is_empty(),
        "{} of 40 freshly installed stand-ins never ran: {unlaunched:#?}",
        unlaunched.len()
    );
}

/// A stand-in for Firefox that occupies a startup for a window it cannot
/// shorten and then exits, having never announced `BiDi`.
fn stand_in_firefox(dir: &Path, window: Duration) -> PathBuf {
    let script = dir.join("stand-in-firefox.sh");
    executable::install_bytes(
        &script,
        format!("#!/bin/sh\nsleep {:.3}\n", window.as_secs_f64()).as_bytes(),
    )
    .unwrap();
    script
}

#[test]
fn fixtures_never_start_more_than_one_firefox_at_a_time() {
    let root = TemporaryDirectory::create("ess-browser-startup-gate").unwrap();
    // An elapsed deadline would hold the lock across the spawn and nothing else,
    // and a fixture that released it there would pass. These starts each occupy
    // the fixture for a real window, so overlapping them is observable twice
    // over: in the peak this process reached, and in the wall clock.
    let window = Duration::from_millis(500);
    let program = stand_in_firefox(&root, window);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let started = Instant::now();
    let mut threads = Vec::new();
    for index in 0..3 {
        let evidence = root.join(index.to_string());
        fs::create_dir_all(&evidence).unwrap();
        let barrier = std::sync::Arc::clone(&barrier);
        let program = program.clone();
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            let Err(refusal) =
                browser::Browser::launch_program(&evidence, program.as_os_str(), window * 40)
            else {
                panic!("a stand-in that never announced BiDi admitted a browser")
            };
            refusal
        }));
    }
    let refusals: Vec<String> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    let elapsed = started.elapsed();
    for refusal in &refusals {
        assert!(
            refusal.starts_with("fixture environment refusal:"),
            "{refusal}"
        );
        assert!(refusal.contains("stage:            exited"), "{refusal}");
    }
    assert_eq!(browser::peak_concurrent_startups(), 1);
    assert!(
        elapsed >= (window * 3).saturating_sub(Duration::from_millis(200)),
        "three {:.3}s startups finished in {:.3}s, so the fixture ran them concurrently",
        window.as_secs_f64(),
        elapsed.as_secs_f64()
    );
}

#[test]
fn a_firefox_this_runner_does_not_have_refuses_rather_than_reading_as_a_defect() {
    let evidence = TemporaryDirectory::create("ess-browser-startup-absent").unwrap();
    let absent = evidence.join("no-such-firefox");
    let Err(refusal) =
        browser::Browser::launch_program(&evidence, absent.as_os_str(), browser::STARTUP_DEADLINE)
    else {
        panic!("a browser that was never spawned admitted a session")
    };
    fs::write(evidence.join("absent-refusal.txt"), &refusal).unwrap();
    assert!(
        refusal.starts_with("fixture environment refusal:"),
        "{refusal}"
    );
    assert!(refusal.contains("not a BiDi protocol defect"), "{refusal}");
    assert!(refusal.contains("stage:            spawn ("), "{refusal}");
    assert!(refusal.contains("os error 2"), "{refusal}");
    // Nothing about a startup was measured here, and the report says that rather
    // than printing a sub-millisecond number beside an untouched deadline.
    assert!(
        refusal.contains("measured startup: not timed; the browser was never spawned"),
        "{refusal}"
    );
    assert!(!refusal.contains("deadline:"), "{refusal}");
}

#[test]
fn browser_displays_logical_namespace_requirement_without_claiming_backend_reset() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence = TemporaryDirectory::create("ess-initial-state-browser").unwrap();
    let generated = evidence.join("site");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .args(["--suite-format", "5", "--out"])
        .arg(&generated)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let server = browser::Server::new(&generated);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(
        &context,
        r"(async()=>{
      await import('./player.js');
      await new Promise(resolve=>setTimeout(resolve,100));
      return JSON.stringify({text:document.body.textContent});
    })()",
    );
    let text = result["text"].as_str().unwrap();
    assert!(
        text.contains("empty logical modeled-instance/event/invocation namespace"),
        "{text}"
    );
    assert!(
        text.contains("does not establish backend isolation"),
        "{text}"
    );
}
