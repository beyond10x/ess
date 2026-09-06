//! The checked generated reader compares full inherited definitions using Rust's typed meaning.

#[test]
fn go_execution_adapts_only_selected_integer_fields_before_target_effects() {
    let dir = std::env::temp_dir().join(format!("ess-coverage-adaptation-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let input = AdmittedInput::from_suite(
        AdmittedSuite::from_json(&coverage_cases::document().to_string()).unwrap(),
    )
    .unwrap();
    for artifact in ess_conformance::go::emit_input(&input).unwrap() {
        let path = dir.join(artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.contents).unwrap();
    }
    fs::write(dir.join("go.mod"), "module coverageadaptation\n\ngo 1.24\n").unwrap();
    fs::write(dir.join("essconform/adaptation_test.go"),r#"package essconform
import ("os";"testing")
type adaptationTarget struct { Target }
func recordAdaptation(value string) {
    file, err := os.OpenFile(os.Getenv("CALLBACKS"),os.O_CREATE|os.O_APPEND|os.O_WRONLY,0600)
    if err != nil { panic(err) }; defer file.Close()
    if _, err := file.WriteString(value+"\n"); err != nil { panic(err) }
}
func (adaptationTarget) Identity() (Identity,error) { recordAdaptation("identity");return Identity{Name:"adaptation-control",Version:"1"},nil }
func (adaptationTarget) BeginScenario(ScenarioContext) error { recordAdaptation("begin");return nil }
func (adaptationTarget) EndScenario(ScenarioContext) error { recordAdaptation("end");return nil }
func TestAdaptation(t *testing.T) {
    original, err := os.ReadFile(os.Getenv("INPUT")); if err != nil { t.Fatal(err) }; suiteJSON=string(original)
    countReportNow=func()int64{return 0}
    Run(t,func()Target { recordAdaptation("factory");return adaptationTarget{} })
}
"#).unwrap();
    let cases = coverage_cases::cases();
    for (name, field) in [
        ("halt-defaults", Some("after")),
        ("count-upper-wire", Some("at_least")),
        ("position-upper-wire", Some("index")),
        ("oversized-parent-only", None),
    ] {
        let case = cases.iter().find(|case| case.name == name).unwrap();
        let input = AdmittedInput::from_json(&case.input).unwrap();
        let run = dir.join(name);
        fs::create_dir_all(&run).unwrap();
        let original = run.join("input.json");
        fs::write(&original, &case.input).unwrap();
        let report = run.join("report.json");
        fs::write(&report, "unchanged\n").unwrap();
        let callbacks = run.join("callbacks.txt");
        let mut command = Command::new("go");
        command
            .args(["test", "-count=1", "-v", "./..."])
            .current_dir(&dir)
            .env("INPUT", &original)
            .env("CALLBACKS", &callbacks)
            .env("ESS_REPORT_OUT", &report)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_CONFORMANCE_STRICT", "1")
            .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE");
        fs::write(run.join("command"), format!("{command:?}\n")).unwrap();
        let output = command.output().unwrap();
        fs::write(run.join("stdout"), &output.stdout).unwrap();
        fs::write(run.join("stderr"), &output.stderr).unwrap();
        fs::write(run.join("exit"), format!("{:?}\n", output.status.code())).unwrap();
        if let Some(field) = field {
            assert!(!output.status.success(), "{name}: {output:?}");
            assert!(
                !callbacks.exists(),
                "{name}: target callback before adaptation"
            );
            assert_eq!(fs::read_to_string(&report).unwrap(), "unchanged\n");
            let diagnostic = String::from_utf8_lossy(&output.stdout);
            assert!(
                diagnostic.contains(coverage_cases::ID),
                "{name}: {diagnostic}"
            );
            assert!(diagnostic.contains(field), "{name}: {diagnostic}");
            assert!(diagnostic.contains("Go int"), "{name}: {diagnostic}");
        } else {
            assert!(output.status.success(), "{name}: {output:?}");
            assert_eq!(
                fs::read_to_string(callbacks).unwrap(),
                "factory\nidentity\nfactory\nbegin\nend\n"
            );
            let count = ess_conformance::CountReport::from_json(
                &fs::read_to_string(report).unwrap(),
                input.selected(),
            )
            .unwrap();
            assert_eq!(count.counts().passed, 1);
            assert_eq!(
                count.conformance_status(),
                ess_conformance::CountStatus::Passed
            );
        }
    }
}
#[path = "support/coverage_cases.rs"]
mod coverage_cases;
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite};
use std::{fs, process::Command};

#[test]
fn generated_go_checks_original_lineage_and_typed_defaults() {
    let cases = coverage_cases::cases();
    for case in &cases {
        let result = AdmittedInput::from_json(&case.input);
        assert_eq!(
            result.is_ok(),
            case.accepted,
            "Rust {}: {result:?}",
            case.name
        );
    }
    let dir = std::env::temp_dir().join(format!("ess-coverage-lineage-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let input = AdmittedInput::from_suite(
        AdmittedSuite::from_json(&coverage_cases::document().to_string()).unwrap(),
    )
    .unwrap();
    for artifact in ess_conformance::go::emit_input(&input).unwrap() {
        let path = dir.join(artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.contents).unwrap();
    }
    fs::write(dir.join("go.mod"), "module coveragelineage\n\ngo 1.24\n").unwrap();
    fs::write(
        dir.join("essconform/cases.json"),
        serde_json::to_string_pretty(&cases).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("essconform/lineage_test.go"),r#"package essconform

import (
    "encoding/json"
    "os"
    "testing"
)

func TestOriginalLineage(t *testing.T) {
    raw, err := os.ReadFile("cases.json")
    if err != nil { t.Fatal(err) }
    var cases []struct { Name string; Input string; Accepted bool }
    if err := json.Unmarshal(raw, &cases); err != nil { t.Fatal(err) }
    for _, test := range cases {
        t.Run(test.Name, func(t *testing.T) {
            _, err := admitRunInput(test.Input)
            if (err == nil) != test.Accepted { t.Fatalf("admitted=%v want=%v: %v", err == nil, test.Accepted, err) }
        })
    }
}
"#).unwrap();
    let mut command = Command::new("go");
    command
        .args(["test", "-count=1", "-v", "./..."])
        .current_dir(&dir);
    fs::write(dir.join("go.command"), format!("{command:?}\n")).unwrap();
    let output = command.output().unwrap();
    fs::write(dir.join("go.stdout"), &output.stdout).unwrap();
    fs::write(dir.join("go.stderr"), &output.stderr).unwrap();
    fs::write(dir.join("go.exit"), format!("{:?}\n", output.status.code())).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
