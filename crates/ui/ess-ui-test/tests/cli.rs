//! The command line: `--path <document> <tests…> [--format text|json] [--playwright <out>]`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use ess_ui_test::TestArgs;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TestArgs,
}

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir exists");
    dir
}

fn args(extra: &[&str]) -> TestArgs {
    let document = example_dir().join("ui.yaml");
    let mut argv = vec![
        "ess-ui-test".to_owned(),
        "--path".to_owned(),
        document.display().to_string(),
    ];
    argv.extend(extra.iter().map(|arg| (*arg).to_owned()));
    Cli::try_parse_from(argv).expect("the arguments parse").args
}

fn failing_file(dir: &Path) -> PathBuf {
    let file = dir.join("mixed.yaml");
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n\
         - name: passes\n  steps:\n  - open: partners.list\n\
         - name: fails\n  steps:\n  - open: partners.list\n  - expect: {{at: pages/partners.list/sections/list, rows: 2}}\n",
        example_dir().join("ui.yaml").display()
    );
    std::fs::write(&file, text).expect("the test file is written");
    file
}

#[test]
fn json_reports_every_test_and_exits_one_on_a_failure() {
    let dir = scratch("json");
    let file = failing_file(&dir);
    let file = file.display().to_string();
    let mut out = Vec::new();
    let code = ess_ui_test::run_to(&args(&[&file, "--format", "json"]), &mut out)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(code, ExitCode::from(1));
    let report: serde_json::Value = serde_json::from_slice(&out).expect("the report is JSON");
    assert_eq!(report["format"], "ess-ui-test-report/1");
    assert!(
        report["document"]
            .as_str()
            .is_some_and(|document| document.ends_with("ui.yaml")),
        "{report}"
    );
    let tests = report["tests"].as_array().expect("tests");
    assert_eq!(tests.len(), 2);
    assert_eq!(tests[0]["name"], "passes");
    assert_eq!(tests[0]["status"], "passed");
    assert!(tests[0].get("step").is_none(), "{report}");
    assert!(tests[0].get("message").is_none(), "{report}");
    assert_eq!(tests[1]["status"], "failed");
    assert_eq!(tests[1]["step"], 2);
    assert!(
        tests[1]["message"]
            .as_str()
            .is_some_and(|message| message.contains("expected 2 rows")),
        "{report}"
    );
}

#[test]
fn text_is_the_default_and_a_passing_run_exits_zero() {
    let tests = example_dir().join("tests").join("stale.yaml");
    let tests = tests.display().to_string();
    let mut out = Vec::new();
    let code =
        ess_ui_test::run_to(&args(&[&tests]), &mut out).unwrap_or_else(|error| panic!("{error}"));
    let text = String::from_utf8(out).expect("utf-8");
    assert_eq!(code, ExitCode::SUCCESS, "{text}");
    assert!(
        text.contains("passed  a stale channel marks its section"),
        "{text}"
    );
    assert!(text.contains("2 passed, 0 failed"), "{text}");

    let dir = scratch("text");
    let file = failing_file(&dir).display().to_string();
    let mut out = Vec::new();
    let code =
        ess_ui_test::run_to(&args(&[&file]), &mut out).unwrap_or_else(|error| panic!("{error}"));
    let text = String::from_utf8(out).expect("utf-8");
    assert_eq!(code, ExitCode::from(1));
    assert!(text.contains("failed  fails (step 2)"), "{text}");
}

#[test]
fn playwright_writes_a_spec_and_runs_nothing() {
    let dir = scratch("playwright");
    let out_file = dir.join("portal.spec.ts");
    let tests = example_dir().join("tests").join("partners-list.yaml");
    let tests = tests.display().to_string();
    let out_arg = out_file.display().to_string();
    let mut out = Vec::new();
    let code = ess_ui_test::run_to(&args(&[&tests, "--playwright", &out_arg]), &mut out)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(code, ExitCode::SUCCESS);
    let spec = std::fs::read_to_string(&out_file).expect("the spec is written");
    assert!(spec.contains("@playwright/test"), "{spec}");
    let said = String::from_utf8(out).expect("utf-8");
    assert!(said.contains("portal.spec.ts"), "{said}");
}

#[test]
fn a_missing_test_file_is_an_error_not_a_report() {
    let error = ess_ui_test::run_to(&args(&["/nonexistent/tests.yaml"]), &mut Vec::new())
        .expect_err("refused");
    assert!(
        error.to_string().contains("/nonexistent/tests.yaml"),
        "{error}"
    );
}
