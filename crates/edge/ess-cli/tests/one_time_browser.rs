//! Disclosure declarations remain visible without claiming browser execution evidence.
#[path = "support/browser.rs"]
mod browser;

use ess_cli::TemporaryDirectory;
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    admission::AdmittedSuite,
    coverage::{Origins, Scope},
    coverage_build, web,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;
use std::{collections::BTreeMap, fs};

const MODEL: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
const SUITE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/one-time-response/valid-string.json"
);

fn model() -> EssIr {
    let specification = Specification::assemble([(
        Source::new("policy.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    compile(&specification, &SourceMap::new()).unwrap()
}

fn browse(name: &str, artifacts: &BTreeMap<String, ess_gen::Artifact>) -> Value {
    let evidence = TemporaryDirectory::create(&format!("ess-one-time-browser-{name}")).unwrap();
    let output_directory = evidence.join("site");
    for artifact in artifacts.values() {
        let path = output_directory.join(&artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &artifact.contents).unwrap();
    }
    let server = browser::Server::new(&output_directory);
    let mut browser = browser::Browser::new(&evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let result = browser.evaluate(
        &context,
        r"(async()=>{
          const {default:p}=await import('./player.js');
          const {nextTick}=await import('./assets/vue.esm-browser.prod.js');
          await nextTick();
          const step=[...document.querySelectorAll('button')].find(b=>b.textContent.trim().startsWith('Step'));
          if(p.scenarios.length) step.click(); await nextTick();
          return JSON.stringify({dom:document.getElementById('app').textContent,
            boom:document.getElementById('boom').textContent,
            origins:p.scenarios[0]?.oneTimeOrigins ?? [],
            steps:p.scenarios.flatMap(s=>s.acts.flatMap(a=>a.steps)),world:p.state.world});
        })()",
    );
    fs::write(evidence.join("result.json"), result.to_string()).unwrap();
    assert_eq!(result["boom"], "", "{result}");
    result
}

#[test]
fn declaration_player_renders_policy_and_keeps_expectations_unexecuted() {
    let model = model();
    let suite = AdmittedSuite::from_json(SUITE).unwrap();
    let artifacts = web::emit(&model, suite.suite()).unwrap();
    let persisted: Value = serde_json::from_str(&artifacts["suite.json"].contents).unwrap();
    let original: Value = serde_json::from_str(SUITE).unwrap();
    assert_eq!(
        persisted, original,
        "the complete closed policy must survive"
    );
    let result = browse("selected", &artifacts);
    let dom = result["dom"].as_str().unwrap();
    for required in [
        "One-time response obligation (unexecuted)",
        "credentials.api.Issue / issued",
        "fields secret",
        "observes no values and verifies no non-disclosure guarantee",
    ] {
        assert!(dom.contains(required), "missing {required}: {dom}");
    }
    assert_eq!(result["origins"][0]["fields"][0], "secret");
    let steps = result["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert!(steps
        .iter()
        .all(|step| step["status"] == "Unexecuted declaration"));
    assert_eq!(result["world"]["instances"], serde_json::json!({}));
    assert_eq!(result["world"]["events"], serde_json::json!([]));
}

#[test]
fn source_policy_remains_visible_when_no_selected_scenario_exercises_it() {
    let model = model();
    let suite = AdmittedSuite::from_json(SUITE).unwrap();
    let mut empty = suite.suite().clone();
    empty.scenarios.clear();
    let artifacts = web::emit(&model, &empty).unwrap();
    let projected: Value = serde_json::from_str(&artifacts["model.json"].contents).unwrap();
    assert_eq!(
        projected["commands"][0]["outcomes"][0]["one_time_response"],
        serde_json::json!(["secret"])
    );
    let result = browse("empty", &artifacts);
    let dom = result["dom"].as_str().unwrap();
    for required in [
        "One-time response source obligations",
        "credentials.api.Issue / issued",
        "fields secret",
        "even when no selected scenario exercises it",
        "verifies no non-disclosure guarantee",
    ] {
        assert!(dom.contains(required), "missing {required}: {dom}");
    }
    assert_eq!(result["steps"], serde_json::json!([]));
    assert_eq!(result["origins"], serde_json::json!([]));
}

#[test]
fn closed_coverage_replay_refuses_policy_before_emitting_an_incomplete_projection() {
    let model = model();
    let input = coverage_build::build(&model, &[], Scope::System, Origins::Generated).unwrap();
    for failure in [
        web::emit_input(&model, &input).unwrap_err(),
        ess_conformance::web_replay::AdmittedReplay::new(&model, &input).unwrap_err(),
    ] {
        assert!(
            failure.issues.iter().any(|issue| {
                issue.reason == "UnsupportedVocabulary"
                    && issue.path == "$model"
                    && issue.detail.contains("one_time_response")
            }),
            "{failure}"
        );
    }
}
