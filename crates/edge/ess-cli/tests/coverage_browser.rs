//! The generated generic player is exercised in actual Firefox, including admission refusal.
#[path = "support/browser.rs"]
mod browser;
#[path = "support/coverage_cases.rs"]
mod coverage_cases;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn actual_browser_and_rust_refuse_every_closed_model_field_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence =
        std::env::temp_dir().join(format!("ess-coverage-browser-model-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
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
    let evidence = std::env::temp_dir().join(format!("ess-legacy-browser-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
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
    // Exercise the immutable historical reader explicitly; fresh players have their own vectors.
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
    assert_eq!(result["scenarios"], 1);
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
    let replay: serde_json::Value =
        serde_json::from_slice(&fs::read(modern.join("replay.json")).unwrap()).unwrap();
    let mut suite: serde_json::Value =
        serde_json::from_str(replay["input"]["suite_json"].as_str().unwrap()).unwrap();
    assert_eq!(suite["provenance"]["suite_version"], "ess-conformance/5");
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
    assert_eq!(result["scenarios"], 1);
    assert_eq!(result["coverageBanner"], false);
}

#[test]
fn actual_browser_admits_the_pair_before_creating_replay_state() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence =
        std::env::temp_dir().join(format!("ess-coverage-browser-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
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
    assert_eq!(result["scenarios"], 1);
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
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-browser-lineage-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
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
    let evidence = std::env::temp_dir().join(format!(
        "ess-browser-startup-deadline-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
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
    let evidence = std::env::temp_dir().join(format!(
        "ess-browser-startup-evidence-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
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

/// The class F1 named: a start this process loses reports through
/// `startup_refusal`, and the only lines on the startup path that may end it any
/// other way are the ones that say which exception they are. The sixth give-up
/// site was found by an adversary because the enumeration behind that claim was
/// a reading of the file; this is the same enumeration, made by the compiler's
/// own copy of the file every time the suite runs.
#[test]
fn no_unaccounted_panic_site_can_end_a_start() {
    // A marker is the whole tail of its line, so the region's own prose can name
    // the markers without opening or closing anything.
    let marks = |line: &str, marker: &str| line.trim_end().ends_with(marker);
    let source = include_str!("support/browser.rs");
    let mut region = Vec::new();
    let mut inside = false;
    for (index, line) in source.lines().enumerate() {
        if marks(line, "// startup-path: begin") {
            inside = true;
        } else if marks(line, "// startup-path: end") {
            inside = false;
        } else if inside {
            region.push((index + 1, line));
        }
    }
    assert!(
        region.len() > 50,
        "the startup path region is not marked in support/browser.rs: {} lines found",
        region.len()
    );
    let forms = [
        ".unwrap()",
        ".expect(",
        "panic!",
        "assert!",
        "assert_eq!",
        "assert_ne!",
        "unreachable!",
        "todo!",
    ];
    let mut accounted = 0;
    let mut previous = "";
    for (number, line) in region {
        if forms.iter().any(|form| line.contains(form)) {
            assert!(
                [line, previous]
                    .iter()
                    .any(|text| marks(text, "// startup-path: harness")
                        || marks(text, "// startup-path: defect")),
                "support/browser.rs:{number} can end a start without a fixture environment \
                 refusal and names no reason. Give up through startup_refusal, or mark the line \
                 `startup-path: harness` (this runner's own filesystem) or `startup-path: defect` \
                 (a deliberate BiDi defect signal):\n{line}"
            );
            accounted += 1;
        }
        previous = line;
    }
    assert!(
        accounted >= 8,
        "the startup path guard matched {accounted} panic sites, so it is no longer reading the \
         fixture it is supposed to hold"
    );
}

#[test]
fn a_browser_that_exits_during_startup_refuses_with_its_own_stderr_too() {
    let evidence =
        std::env::temp_dir().join(format!("ess-browser-startup-exit-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
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

/// A stand-in for Firefox that occupies a startup for a window it cannot
/// shorten and then exits, having never announced `BiDi`.
fn stand_in_firefox(dir: &Path, window: Duration) -> PathBuf {
    let script = dir.join("stand-in-firefox.sh");
    fs::write(
        &script,
        format!("#!/bin/sh\nsleep {:.3}\n", window.as_secs_f64()),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    script
}

#[test]
fn fixtures_never_start_more_than_one_firefox_at_a_time() {
    let root =
        std::env::temp_dir().join(format!("ess-browser-startup-gate-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
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
    let evidence =
        std::env::temp_dir().join(format!("ess-browser-startup-absent-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
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
