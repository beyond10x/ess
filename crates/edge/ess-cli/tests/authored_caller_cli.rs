//! `ess verify conform author` and `run` over an authored act that states its caller (`caller:`,
//! `ess-scenario/5`): the consumer's shape, in neutral names, authors with no refusal and passes on
//! the interpreted target; an act that leaves the attribute its command reads unstated is refused
//! naming both; and a suite whose step carries no caller is `unsupported` with a reason naming the
//! attribute, in the text and the JSON report.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/authored-caller.yaml");
const SCENARIO: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/authored-caller-scenario.yaml");
const SCENARIO_ID: &str = "ledger.notes/authored/a-note-carries-the-callers-account";

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

struct Workspace {
    _dir: tempfile::TempDir,
    model: std::path::PathBuf,
    scenarios: std::path::PathBuf,
    suite: std::path::PathBuf,
}

fn workspace(scenario: &str) -> Workspace {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("ledger.yaml");
    let scenarios = dir.path().join("scenarios");
    fs::create_dir(&scenarios).unwrap();
    fs::write(&model, MODEL).unwrap();
    fs::write(scenarios.join("note.yaml"), scenario).unwrap();
    let suite = dir.path().join("suite.json");
    Workspace {
        model,
        scenarios,
        suite,
        _dir: dir,
    }
}

fn author(workspace: &Workspace) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "author", "--path"])
        .arg(&workspace.model)
        .arg("--scenarios")
        .arg(&workspace.scenarios)
        .arg("--out")
        .arg(&workspace.suite)
        .output()
        .unwrap()
}

fn run(workspace: &Workspace, suite: &Path, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "run", "--target", "interpreted"])
        .args(["--report-format", "2", "--format", format, "--path"])
        .arg(&workspace.model)
        .arg("--suite")
        .arg(suite)
        .output()
        .unwrap()
}

#[test]
fn an_authored_caller_authors_cleanly_and_passes_on_the_interpreted_target() {
    let workspace = workspace(SCENARIO);
    let authored = author(&workspace);
    assert!(authored.status.success(), "{}", text(&authored));
    assert!(
        text(&authored).contains("1 authored scenario(s) from 1 file(s), 0 refusal(s)"),
        "{}",
        text(&authored)
    );
    let suite = fs::read_to_string(&workspace.suite).unwrap();
    assert!(suite.contains("\"caller\""), "{suite}");

    let ran = run(&workspace, &workspace.suite, "text");
    assert!(ran.status.success(), "{}", text(&ran));
    assert!(
        text(&ran).contains(&format!("passed {SCENARIO_ID}")),
        "{}",
        text(&ran)
    );
}

#[test]
fn an_act_leaving_a_read_attribute_unstated_is_refused_naming_it_and_the_command() {
    let unstated = SCENARIO
        .replace("    caller: {account_id: {$instance: account}}\n", "")
        .replace(
            "    caller: {account_id: 3f1d5b7e-0000-4000-8000-000000000002}\n",
            "",
        );
    let workspace = workspace(&unstated);
    let authored = author(&workspace);
    assert!(!authored.status.success(), "{}", text(&authored));
    let printed = text(&authored);
    assert!(printed.contains("ESS-AUTHOR-014"), "{printed}");
    for named in [
        "`account_id`",
        "`ledger.notes.OpenNote`",
        "`ledger.notes.Member`",
    ] {
        assert!(printed.contains(named), "{named}: {printed}");
    }
}

#[test]
fn a_step_sent_as_no_caller_is_unsupported_with_a_reason_naming_the_attribute() {
    let workspace = workspace(SCENARIO);
    let authored = author(&workspace);
    assert!(authored.status.success(), "{}", text(&authored));
    // A suite whose command steps carry no caller, as one written before `caller:` existed.
    let mut suite: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&workspace.suite).unwrap()).unwrap();
    let mut cleared = 0;
    for scenario in suite["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step.as_object_mut().unwrap().remove("caller").is_some() {
                cleared += 1;
            }
        }
    }
    assert_eq!(cleared, 2);
    let bare = workspace.suite.with_file_name("bare.json");
    fs::write(&bare, serde_json::to_string_pretty(&suite).unwrap()).unwrap();

    for format in ["text", "json"] {
        let ran = run(&workspace, &bare, format);
        assert!(!ran.status.success(), "{format}: {}", text(&ran));
        let printed = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(printed.contains("unsupported"), "{format}: {printed}");
        assert!(
            printed.contains("reads caller attribute `account_id`"),
            "{format}: {printed}"
        );
        assert!(printed.contains("`caller:`"), "{format}: {printed}");
    }
}
