//! Actual opt-in CLI production and fail-before-output pairing.
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn recorded(dir: &Path, name: &str, args: &[&str]) -> Output {
    fs::write(
        dir.join(format!("{name}.command")),
        serde_json::to_string_pretty(args).unwrap(),
    )
    .unwrap();
    let output = command(args);
    fs::write(dir.join(format!("{name}.stdout")), &output.stdout).unwrap();
    fs::write(dir.join(format!("{name}.stderr")), &output.stderr).unwrap();
    fs::write(
        dir.join(format!("{name}.exit")),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    output
}
#[test]
fn coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths()
{
    let dir = directory("source-paths");
    let model = root().join("examples/billing");
    let source = include_str!("fixtures/coverage-producers/inputs/authored/single/a.yaml");
    let mut originals = Vec::new();
    for name in ["first", "relocated"] {
        let path = dir.join(name);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("a.yaml"), source).unwrap();
        let output = recorded(
            &dir,
            name,
            &[
                "conform",
                "author",
                "--path",
                model.to_str().unwrap(),
                "--scenarios",
                path.to_str().unwrap(),
                "--suite-format",
                "5",
                "--format",
                "json",
            ],
        );
        assert!(output.status.success(), "{output:?}");
        originals.push(output.stdout);
    }
    assert_eq!(originals[0], originals[1]);
    let input = AdmittedSuite::from_json(std::str::from_utf8(&originals[0]).unwrap()).unwrap();
    assert_eq!(
        input
            .coverage()
            .unwrap()
            .authored_sources
            .keys()
            .next()
            .unwrap()
            .as_str(),
        "a.yaml"
    );
    let path = dir.join("relocated/a.yaml");
    fs::write(&path, source.replace('\n', "\r\n")).unwrap();
    let changed = recorded(
        &dir,
        "changed-newlines",
        &[
            "conform",
            "author",
            "--path",
            model.to_str().unwrap(),
            "--scenarios",
            path.to_str().unwrap(),
            "--suite-format",
            "5",
            "--format",
            "json",
        ],
    );
    assert!(changed.status.success(), "{changed:?}");
    let changed = AdmittedSuite::from_json(std::str::from_utf8(&changed.stdout).unwrap()).unwrap();
    assert_eq!(input.suite().scenarios, changed.suite().scenarios);
    assert_ne!(input.digest(), changed.digest());
    let destination = dir.join("refused.json");
    fs::write(&destination, "unchanged\n").unwrap();
    for (index, path) in invalid_source_paths(&dir, source).iter().enumerate() {
        let output = recorded(
            &dir,
            &format!("refused-{index}"),
            &[
                "conform",
                "author",
                "--path",
                model.to_str().unwrap(),
                "--scenarios",
                path.to_str().unwrap(),
                "--suite-format",
                "5",
                "--format",
                "json",
                "--out",
                destination.to_str().unwrap(),
            ],
        );
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "unchanged\n");
    }
}
fn invalid_source_paths(dir: &Path, source: &str) -> Vec<PathBuf> {
    use std::os::unix::{ffi::OsStringExt, fs::symlink};
    let symlink_root = dir.join("linked-root");
    symlink(dir.join("first"), &symlink_root).unwrap();
    let symlink_file = dir.join("linked.yaml");
    symlink(dir.join("first/a.yaml"), &symlink_file).unwrap();
    let empty = dir.join("empty");
    fs::create_dir_all(&empty).unwrap();
    let mut bad = vec![symlink_root, symlink_file, empty];
    for (index, name) in [
        b"bad:name.yaml".to_vec(),
        b"bad\\name.yaml".to_vec(),
        b"bad\nname.yaml".to_vec(),
        b"\xff.yaml".to_vec(),
    ]
    .into_iter()
    .enumerate()
    {
        let path = dir.join(format!("invalid-{index}"));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join(std::ffi::OsString::from_vec(name)), source).unwrap();
        bad.push(path);
    }
    bad
}
#[test]
fn coverage_cli_refuses_binary64_model_before_each_new_production_surface() {
    let dir = directory("binary64");
    let model = dir.join("model");
    fs::create_dir_all(&model).unwrap();
    fs::write(model.join("system.yaml"),"format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.data]\ndomain: sample.data\ntypes:\n  - {name: sample.data.Ratio, kind: newtype, of: Binary64}\n").unwrap();
    let scenarios = dir.join("scenarios");
    fs::create_dir_all(&scenarios).unwrap();
    fs::write(
        scenarios.join("a.yaml"),
        include_str!("fixtures/coverage-producers/inputs/authored/single/a.yaml"),
    )
    .unwrap();
    for route in ["synthesize", "author", "web", "run"] {
        let destination = dir.join(format!("{route}-out"));
        let mut args = vec![
            "conform",
            route,
            "--path",
            model.to_str().unwrap(),
            "--suite-format",
            "5",
        ];
        if route == "run" {
            args.extend([
                "--target",
                "billing",
                "--report-format",
                "2",
                "--report-out",
                destination.to_str().unwrap(),
            ]);
        } else {
            args.extend(["--out", destination.to_str().unwrap()]);
        }
        if route == "author" || route == "web" {
            args.extend(["--scenarios", scenarios.to_str().unwrap()]);
        }
        let output = recorded(&dir, route, &args);
        assert!(!output.status.success(), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Binary64"),
            "{output:?}"
        );
        assert!(!destination.exists());
    }
    let destination = dir.join("go-out");
    let output = recorded(
        &dir,
        "go",
        &[
            "conform",
            "synthesize",
            "--path",
            model.to_str().unwrap(),
            "--suite-format",
            "5",
            "--target",
            "go",
            "--out",
            destination.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success(), "{output:?}");
    assert!(!destination.exists());
}
fn directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("ess-coverage-cli-{name}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}
fn generated(directory: &Path) -> PathBuf {
    let suite = directory.join("suite.json");
    let output = command(&[
        "conform",
        "synthesize",
        "--path",
        root().join("examples/billing").to_str().unwrap(),
        "--suite-format",
        "5",
        "--out",
        suite.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, fs::read(&suite).unwrap());
    suite
}
#[test]
fn explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution() {
    let dir = directory("pairing");
    let suite = generated(&dir);
    let original = fs::read_to_string(&suite).unwrap();
    let admitted = AdmittedSuite::from_json(&original).unwrap();
    assert_eq!(admitted.coverage().unwrap().generated.len(), 29);
    let destination = dir.join("report.json");
    fs::write(&destination, "unchanged\n").unwrap();
    for extra in [
        vec![],
        vec!["--allow-incomplete"],
        vec!["--report-format", "1"],
    ] {
        let mut args = vec![
            "conform",
            "run",
            "--suite",
            suite.to_str().unwrap(),
            "--target",
            "billing",
            "--report-out",
            destination.to_str().unwrap(),
        ];
        args.extend(extra);
        let output = command(&args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "unchanged\n");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("report"),
            "{output:?}"
        );
    }
    let output = command(&[
        "conform",
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--target",
        "billing",
        "--report-format",
        "2",
        "--strict",
        "--format",
        "json",
        "--report-out",
        destination.to_str().unwrap(),
    ]);
    assert!(output.status.success(), "{output:?}");
    let run: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(run["summary"]["conformance_status"], "passed");
    assert_eq!(run["summary"]["counts"]["total"], 29);
    ess_conformance::CountReport::from_json(&fs::read_to_string(destination).unwrap(), &admitted)
        .unwrap();
}
#[test]
fn select_cli_preserves_all_parent_bytes_and_explicit_empty_selection() {
    let dir = directory("selection");
    let suite = generated(&dir);
    let ids = dir.join("ids.json");
    let output = dir.join("input.json");
    fs::write(&ids, "[]\n").unwrap();
    let result = command(&[
        "conform",
        "select",
        "--suite",
        suite.to_str().unwrap(),
        "--ids",
        ids.to_str().unwrap(),
        "--out",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{result:?}");
    let original = fs::read_to_string(&output).unwrap();
    let admitted = AdmittedInput::from_json(&original).unwrap();
    assert_eq!(
        admitted.parents()[0].original_json(),
        fs::read_to_string(&suite).unwrap()
    );
    assert_eq!(admitted.selected().coverage().unwrap().counts.outside, 29);
    let run = command(&[
        "conform",
        "run",
        "--suite-input",
        output.to_str().unwrap(),
        "--target",
        "billing",
        "--report-format",
        "2",
        "--strict",
        "--format",
        "json",
    ]);
    assert_eq!(run.status.code(), Some(3), "{run:?}");
    let detailed: Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(detailed["summary"]["counts"]["total"], 0);
    assert_eq!(detailed["summary"]["execution_status"], "passed");
    fs::write(&ids, json!(["billing.invoice/authored/absent"]).to_string()).unwrap();
    let result = command(&[
        "conform",
        "select",
        "--suite-input",
        output.to_str().unwrap(),
        "--ids",
        ids.to_str().unwrap(),
        "--out",
        output.to_str().unwrap(),
    ]);
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(&output).unwrap(), original);
}

#[test]
fn generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults() {
    let dir = directory("go");
    let emitted = command(&[
        "conform",
        "synthesize",
        "--path",
        root().join("examples/billing").to_str().unwrap(),
        "--suite-format",
        "5",
        "--target",
        "go",
        "--out",
        dir.to_str().unwrap(),
    ]);
    fs::write(dir.join("emit.stdout"), &emitted.stdout).unwrap();
    fs::write(dir.join("emit.stderr"), &emitted.stderr).unwrap();
    assert!(emitted.status.success(), "{emitted:?}");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-billing");
    for name in ["target.go", "target_test.go"] {
        fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    fs::write(dir.join("go.mod"), "module essbilling\n\ngo 1.24\n").unwrap();
    let destination = dir.join("report.json");
    fs::write(&destination, "unchanged\n").unwrap();
    for (name, report2, success) in [("default", false, false), ("coverage", true, true)] {
        let mut command = Command::new("go");
        command
            .args(["test", "-count=1", "-v", "./..."])
            .current_dir(&dir)
            .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
            .env_remove("ESS_CONFORMANCE_STRICT")
            .env_remove("ESS_REPORT_FORMAT")
            .env("ESS_REPORT_OUT", &destination);
        if report2 {
            command
                .env("ESS_REPORT_FORMAT", "2")
                .env("ESS_CONFORMANCE_STRICT", "1");
        }
        fs::write(
            dir.join(format!("{name}.command")),
            format!("{command:?}\n"),
        )
        .unwrap();
        let output = command.output().expect("the required Go toolchain runs");
        fs::write(dir.join(format!("{name}.stdout")), &output.stdout).unwrap();
        fs::write(dir.join(format!("{name}.stderr")), &output.stderr).unwrap();
        fs::write(
            dir.join(format!("{name}.exit")),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
        assert_eq!(output.status.success(), success, "{output:?}");
        if report2 {
            let input = AdmittedInput::from_json(
                &fs::read_to_string(dir.join("essconform/input.json")).unwrap(),
            )
            .unwrap();
            let report = ess_conformance::CountReport::from_json(
                &fs::read_to_string(&destination).unwrap(),
                input.selected(),
            )
            .unwrap();
            assert_eq!(report.counts().total, 29);
            assert_eq!(report.counts().passed, 29);
            assert_eq!(
                report.conformance_status(),
                ess_conformance::CountStatus::Passed
            );
        } else {
            assert_eq!(fs::read_to_string(&destination).unwrap(), "unchanged\n");
        }
    }
}

#[test]
fn impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape() {
    let dir = directory("impact");
    let path = generated(&dir);
    let all = AdmittedInput::from_suite(
        AdmittedSuite::from_json(&fs::read_to_string(&path).unwrap()).unwrap(),
    )
    .unwrap();
    let input = all.select(&[]).unwrap();
    let carrier = dir.join("input.json");
    fs::write(&carrier, input.document().to_canonical_json().unwrap()).unwrap();
    let model = root().join("examples/billing");
    let output = command(&[
        "impact",
        "--from",
        model.to_str().unwrap(),
        "--to",
        model.to_str().unwrap(),
        "--suite-input",
        carrier.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-impact/3");
    assert!(report.get("selection").is_none() && report.get("coverage").is_none());
    assert!(String::from_utf8_lossy(&output.stderr).contains("selection"));
    let mut unknown: Value = serde_json::from_str(all.selected().original_json()).unwrap();
    unknown["coverage"]["knowledge"] = json!("unknown");
    fs::write(&path, unknown.to_string()).unwrap();
    let output = command(&[
        "impact",
        "--from",
        model.to_str().unwrap(),
        "--to",
        model.to_str().unwrap(),
        "--suite",
        path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
