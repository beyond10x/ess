//! The generated generic player is exercised in actual Firefox, including admission refusal.
#[path = "support/browser.rs"]
mod browser;
#[path = "support/coverage_cases.rs"]
mod coverage_cases;
use std::{fs, path::Path, process::Command};

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
          const suite = {version:'ess-conformance/5',digest_profile:'sha256-json-bytes/1',digest};
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
