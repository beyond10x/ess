//! Public original-input and actual browser regression cases for the frozen coverage writer.
#[path = "support/browser.rs"]
mod browser;

use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

#[test]
fn go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown() {
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-adversary-go-diagnostic-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut emit = Command::new(env!("CARGO_BIN_EXE_ess"));
    emit.args(["conform", "author", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/coverage-producers/inputs/authored/single"),
        )
        .args(["--suite-format", "5", "--format", "json"]);
    fs::write(evidence.join("emit.command"), format!("{emit:?}\n")).unwrap();
    let emitted = emit.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &emitted.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &emitted.stderr).unwrap();
    fs::write(
        evidence.join("emit.exit"),
        format!("{:?}\n", emitted.status.code()),
    )
    .unwrap();
    assert!(emitted.status.success(), "{emitted:?}");
    let suite =
        ess_conformance::AdmittedSuite::from_json(std::str::from_utf8(&emitted.stdout).unwrap())
            .unwrap();
    assert!(suite.coverage().unwrap().is_complete());
    assert_eq!(suite.suite().len(), 1);
    let input = ess_conformance::coverage::AdmittedInput::from_suite(suite).unwrap();
    for artifact in ess_conformance::go::emit_input(&input).unwrap() {
        let path = evidence.join(artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.contents).unwrap();
    }
    fs::write(evidence.join("go.mod"), "module diagnostic\n\ngo 1.24\n").unwrap();
    fs::write(evidence.join("essconform/diagnostic_test.go"), r#"package essconform
import ("testing")
type unavailableTarget struct { Target }
func (unavailableTarget) Identity() (Identity,error) { return Identity{Name:"unavailable",Version:"1"},nil }
func (unavailableTarget) BeginScenario(ScenarioContext) error { return ErrUnsupported }
func TestStrictDiagnostic(t *testing.T) {
    countReportNow=func()int64{return 0}
    Run(t,func()Target{return unavailableTarget{}})
}
"#).unwrap();
    let report = evidence.join("report.json");
    let mut command = Command::new("go");
    command
        .args(["test", "-count=1", "-v", "./..."])
        .current_dir(&evidence)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_CONFORMANCE_STRICT", "1")
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env("ESS_REPORT_OUT", &report);
    fs::write(evidence.join("go.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("go.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("go.stderr"), &output.stderr).unwrap();
    fs::write(
        evidence.join("go.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(!output.status.success(), "strict skipped run must fail");
    let original_report = fs::read_to_string(report).unwrap();
    let count =
        ess_conformance::CountReport::from_json(&original_report, input.selected()).unwrap();
    assert_eq!(count.counts().total, 1);
    assert_eq!(count.counts().skipped, 1);
    assert_eq!(
        count.conformance_status(),
        ess_conformance::CountStatus::Inconclusive
    );
    let wire: Value = serde_json::from_str(&original_report).unwrap();
    assert_eq!(wire["coverage"]["knowledge"], "complete_inventory");
    let diagnostic = String::from_utf8(output.stdout).unwrap();
    println!("{diagnostic}");
    assert!(
        !diagnostic.contains("legacy suite coverage is unknown"),
        "suite/5 diagnostic contradicts its admitted known inventory: {diagnostic}"
    );
}

#[test]
fn browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-adversary-final-lf-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
    let site = evidence.join("site");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .args(["--suite-format", "5", "--out"])
        .arg(&site);
    fs::write(evidence.join("emit.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
    fs::write(
        evidence.join("emit.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    let original = fs::read_to_string(site.join("replay.json")).unwrap();
    ess_conformance::web_replay::AdmittedReplay::from_json(&original).unwrap();
    fs::write(evidence.join("original-replay.json"), &original).unwrap();
    let server = browser::Server::new(&site);
    let mut firefox = browser::Browser::new(&evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    let prepared = firefox.evaluate(&context, r"(async () => {
      const {default:player}=await import('./player.js');
      const pair=await fetch('replay.json').then(r=>r.json());
      const suite=JSON.parse(pair.input.suite_json);
      const scenario=Object.values(suite.scenarios)[0];
      const step=scenario.steps.find(s=>s.step==='execute_command');
      const before=step.command;
      step.command += '\n';
      pair.input.suite_json=JSON.stringify(suite);
      pair.suite.digest='sha256:'+Array.from(new Uint8Array(await crypto.subtle.digest(
        'SHA-256',new TextEncoder().encode(pair.input.suite_json))),b=>b.toString(16).padStart(2,'0')).join('');
      return JSON.stringify({originalCursor:player.state.cursor,before,after:step.command,pair});
    })()");
    fs::write(
        evidence.join("prepared.json"),
        serde_json::to_string_pretty(&prepared).unwrap(),
    )
    .unwrap();
    assert_eq!(prepared["originalCursor"], -1);
    assert_eq!(
        prepared["after"],
        format!("{}\n", prepared["before"].as_str().unwrap())
    );
    let changed = prepared["pair"].to_string();
    let refusal = ess_conformance::web_replay::AdmittedReplay::from_json(&changed)
        .expect_err("a control character cannot be part of a command name");
    fs::write(evidence.join("rust-refusal.txt"), format!("{refusal}\n")).unwrap();
    fs::write(evidence.join("invalid-replay.json"), &changed).unwrap();
    fs::write(site.join("replay.json"), &changed).unwrap();
    let context = firefox.open(&format!("{}/index.html?invalid-name", server.url));
    let result: Value = firefox.evaluate(
        &context,
        r"(async () => {
      try {
        const {default:player}=await import('./player.js');
        return JSON.stringify({admitted:true,cursor:player.state.cursor,
          mounted:document.getElementById('app').hasAttribute('data-v-app'),
          banner:document.getElementById('coverage')!==null});
      } catch(error) {
        return JSON.stringify({admitted:false,error:String(error),
          mounted:document.getElementById('app').hasAttribute('data-v-app'),
          banner:document.getElementById('coverage')!==null});
      }
    })()",
    );
    fs::write(
        evidence.join("firefox-result.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("Rust refusal: {refusal}; Firefox: {result}");
    assert_eq!(
        result["admitted"],
        json!(false),
        "invalid command name reached replay: {result}"
    );
    assert_eq!(result["mounted"], false);
    assert_eq!(result["banner"], false);
}
