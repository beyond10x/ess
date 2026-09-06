//! Combined CLI and generated Go count publication refuses Binary64 before effects.

use ess_conformance::ConformanceSuite;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ID: &str = "probe.data/authored/guard";
fn document(binary64: Option<bool>) -> Value {
    let steps = binary64.map_or_else(|| json!([]), |optional| json!([
        {"step":"expect_event","event":"probe.data.Created","shape":{"ratio/a~b":{"holds":"primitive","kind":"binary64","optional":optional}}}
    ]));
    json!({"provenance":{"suite_version":"ess-conformance/4","system":"probe","specification_version":"v1","spec_digest":"a".repeat(64),"contract_digest":"a".repeat(64)},
        "scenarios":{ID:{"purpose":"Refuse Binary64 before report publication","steps":steps,"source":[]}}})
}
fn scratch(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "binary64-count-adversary-{label}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
fn record(root: &Path, label: &str, command: &Command, result: &Output) -> String {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    fs::write(
        root.join(format!("{label}.log")),
        format!("{command:?}\nexit {}\n{text}", result.status),
    )
    .unwrap();
    text
}

#[test]
fn cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats() {
    let root = scratch("cli");
    fs::create_dir_all(root.join("model")).unwrap();
    fs::write(root.join("model/system.yaml"),"format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.data]\ndomain: probe.data\ntypes:\n  - {name: probe.data.Ratio, kind: newtype, of: Binary64}\n").unwrap();
    fs::write(root.join("suite.json"), document(Some(true)).to_string()).unwrap();
    for format in ["1", "2"] {
        for model in [false, true] {
            for existing in [false, true] {
                let label = format!("format-{format}-model-{model}-existing-{existing}");
                let destination = root.join(format!("{label}.report.json"));
                if existing {
                    fs::write(&destination, b"owned report bytes\n").unwrap();
                }
                let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                command
                    .current_dir(&root)
                    .args([
                        "verify",
                        "conform",
                        "run",
                        "--target",
                        "billing",
                        "--report-format",
                        format,
                        "--allow-incomplete",
                        "--format",
                        "json",
                    ])
                    .args(if model {
                        ["--path", "model"]
                    } else {
                        ["--suite", "suite.json"]
                    })
                    .arg("--report-out")
                    .arg(&destination);
                let output = command.output().unwrap();
                let text = record(&root, &label, &command, &output);
                assert!(!output.status.success(), "{label}: {text}");
                assert!(text.contains("Binary64"), "{label}: {text}");
                if model {
                    assert!(text.contains("probe.data.Ratio"), "{text}");
                }
                if existing {
                    assert_eq!(fs::read(&destination).unwrap(), b"owned report bytes\n");
                } else {
                    assert!(!destination.exists(), "{label} published a report");
                }
                eprintln!(
                    "{label}: actual CLI exit {}; destination preserved",
                    output.status
                );
            }
        }
    }
}

fn invoke_go(root: &Path, label: &str, format: &str, marker: &Path, destination: &Path) -> Output {
    let compiler = std::env::var_os("ESS_GO_COMPILER").map_or_else(
        || PathBuf::from("go"),
        |path| {
            let compiler = PathBuf::from(path);
            assert!(compiler.is_absolute() && compiler.is_file());
            compiler
        },
    );
    let mut command = Command::new(compiler);
    command
        .args([
            "test",
            "-v",
            "-count=1",
            "-race",
            "-mod=readonly",
            "./...",
            "-run",
            "^TestGuard$",
        ])
        .current_dir(root)
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOFLAGS", "")
        .env("GOMAXPROCS", "4")
        .env("ESS_REPORT_FORMAT", format)
        .env_remove("ESS_CONFORMANCE_STRICT")
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env("ESS_REPORT_OUT", destination)
        .env("GUARD_MARKER", marker)
        .env(
            "GOCACHE",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-count-adversary-go-cache"),
        );
    let result = command.output().unwrap();
    record(root, label, &command, &result);
    result
}

#[test]
fn generated_go_binary64_shapes_refuse_before_factory_and_report_publication() {
    let root = scratch("go");
    let control = document(None).to_string();
    let suite = ConformanceSuite::from_json(&control).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.contents).unwrap();
    }
    fs::write(
        root.join("go.mod"),
        "module example.invalid/binary64-count-adversary\n\ngo 1.24\n",
    )
    .unwrap();
    fs::write(root.join("essconform/binary64_count_adversary_test.go"),r#"package essconform
import ("os"; "testing")
type guardTarget struct { Target }
func (guardTarget) Identity() (Identity,error) { return Identity{Name:"guard",Version:"1"},nil }
func (guardTarget) BeginScenario(ScenarioContext) error { return nil }
func (guardTarget) EndScenario(ScenarioContext) error { return nil }
func TestGuard(t *testing.T) { Run(t,func() Target {
    if err:=os.WriteFile(os.Getenv("GUARD_MARKER"),[]byte("factory called\n"),0600);err!=nil { panic(err) }
    return guardTarget{}
}) }
"#).unwrap();
    for format in ["1", "2"] {
        fs::write(root.join("essconform/suite.json"), &control).unwrap();
        let label = format!("control-{format}");
        let marker = root.join(format!("{label}.marker"));
        let destination = root.join(format!("{label}.report.json"));
        let output = invoke_go(&root, &label, format, &marker, &destination);
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(&marker).unwrap(), b"factory called\n");
        let report: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
        assert_eq!(report["format"], format!("ess-conformance-report/{format}"));
        eprintln!("{label}: actual Go exit {}; control reached factory and published selected report format",output.status);
        for optional in [false, true] {
            fs::write(
                root.join("essconform/suite.json"),
                document(Some(optional)).to_string(),
            )
            .unwrap();
            for existing in [false, true] {
                let label = format!("format-{format}-optional-{optional}-existing-{existing}");
                let marker = root.join(format!("{label}.marker"));
                let destination = root.join(format!("{label}.report.json"));
                if existing {
                    fs::write(&destination, b"owned report bytes\n").unwrap();
                }
                let output = invoke_go(&root, &label, format, &marker, &destination);
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(!output.status.success(), "{label}: {text}");
                assert!(
                    text.contains("suite admission") && text.contains("unknown primitive"),
                    "{label}: {text}"
                );
                assert!(!marker.exists(), "{label} reached target factory");
                if existing {
                    assert_eq!(fs::read(&destination).unwrap(), b"owned report bytes\n");
                } else {
                    assert!(!destination.exists(), "{label} published report");
                }
                eprintln!(
                    "{label}: actual Go exit {}; factory untouched and destination preserved",
                    output.status
                );
            }
        }
    }
}
