//! beyond10x/ess#330: `ess ui test --model` runs a document whose choice `options` name a model
//! enum with that enum's variants. Without a model the run still starts, and a test fails at the
//! step that shows such a choice, naming the `--model` flag `ess ui test` takes.

use std::path::{Path, PathBuf};

use clap::Parser;
use ess_ui_test::{Status, TestArgs};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TestArgs,
}

const DOCUMENT: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.list
  sections: [{name: all, pages: [releases.list, releases.risk]}]
pages:
  releases.list:
    kind: detail_page
    title: Releases
    sections:
      - {name: summary, component: collection, reads: {placeholder: release.Repositories, fixture: repositories.yaml}, columns: [location]}
  releases.risk:
    kind: detail_page
    title: Risk
    state:
      risk: {type: string, class: page_state, store: url}
    sections:
      - {name: risk, component: choice, options: objective.RiskLevel, binds: state.risk}
";

const FIXTURE: &str =
    "view: release.Repositories\nrows:\n  - {repository_id: repo-1, location: example/one}\n";

const TESTS: &str = "- name: risk\n  steps:\n\
                     \x20 - open: releases.risk\n\
                     \x20 - expect: {at: pages/releases.risk/sections/risk, text: Highest}\n";

/// A directory of its own for test `name`: tests run in parallel and must not rewrite each
/// other's files.
fn dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-model-enums")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("repositories.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    let tests = format!("format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n{TESTS}");
    std::fs::write(dir.join("risk.yaml"), tests).expect("the tests are written");
    dir
}

/// The model as a renderer holds it: the binding `--model` computes, carrying the enum.
fn binding() -> ess_ui::binding::Binding {
    serde_json::from_str(
        r#"{"system": "release", "components": {},
            "names": {"objective.RiskLevel": "release.objective.RiskLevel"},
            "enums": {"release.objective.RiskLevel": [
              {"value": "low", "label": "Lowest"}, {"value": "high", "label": "Highest"}]}}"#,
    )
    .expect("the binding reads")
}

fn files(dir: &Path) -> Vec<ess_ui_test::TestFile> {
    vec![ess_ui_test::load(&dir.join("risk.yaml")).unwrap_or_else(|error| panic!("{error}"))]
}

#[test]
fn with_a_model_the_choice_lists_the_enums_variants() {
    let dir = dir("with");
    let report = ess_ui_test::execute_with(&dir.join("ui.yaml"), &files(&dir), Some(&binding()))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.tests[0].status, Status::Passed, "{report:?}");
}

#[test]
fn without_a_model_the_step_showing_the_choice_fails_naming_model() {
    let dir = dir("without");
    let report = ess_ui_test::execute(&dir.join("ui.yaml"), &files(&dir))
        .unwrap_or_else(|error| panic!("{error}"));
    let outcome = &report.tests[0];
    assert_eq!(outcome.status, Status::Failed, "{report:?}");
    assert_eq!(outcome.step, Some(1), "{outcome:?}");
    let message = outcome.message.as_deref().unwrap_or_default();
    assert!(
        message.contains("pages/releases.risk/sections/risk/options")
            && message.contains("objective.RiskLevel")
            && message.contains("ess ui test --model"),
        "{message}"
    );
}

#[test]
fn the_command_line_takes_model_and_refuses_it_uncompiled() {
    let dir = dir("command-line");
    let path = dir.join("ui.yaml");
    let tests = dir.join("risk.yaml");
    let args = Cli::parse_from([
        "test",
        "--path",
        path.to_str().expect("UTF-8"),
        "--model",
        "spec",
        tests.to_str().expect("UTF-8"),
    ])
    .args;
    assert_eq!(args.model.as_deref(), Some(Path::new("spec")));
    let error = ess_ui_test::run_to(&args, &mut Vec::new()).expect_err("no model was compiled");
    assert!(error.to_string().contains("--model spec"), "{error}");
    let mut out = Vec::new();
    let code = ess_ui_test::run_to_with(&args, &mut out, Some(&binding()))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        code,
        std::process::ExitCode::SUCCESS,
        "{}",
        String::from_utf8_lossy(&out)
    );
}
