//! `--known-failing` on `ess verify conform run`, `report` and `mutate`, against actual runs
//! (beyond10x/ess#294, beyond10x/ess#296; `docs/design/mutation-scope-and-known-failures.md`).
//!
//! Every report here is written by a real runner: the native `ess` runner against the billing
//! reference, the generated Go runner against `fixtures/go-billing` (with its `reversed-order`
//! defect), and the generated Go and TypeScript runners against a small target that fails two
//! scenarios and cannot answer a third. Known-failure accounting is a separate document: the
//! ordinary report keeps its bytes and verdict, and every exit status means what it meant.
//!
//! Skipped, and said out loud, where the machine has no `go`, or no `tsc` and `node`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_conformance::known_failures::{sha256, Accounting, ExecutionContext};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, CountReport, Runner};
use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("known-failures-cli")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the ess binary runs")
}

fn text(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn s(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn tool(name: &str) -> bool {
    let found = Command::new(name)
        .arg(if name == "go" { "version" } else { "--version" })
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `{name}` on PATH, so this case was not run");
    }
    found
}

/// The build every generated runner here is said to be, by its host.
fn host_build() -> String {
    format!("sha256:{}", "b".repeat(64))
}

/// The running `ess` executable's own build identity, as the CLI computes it.
fn ess_build() -> String {
    sha256(&std::fs::read(env!("CARGO_BIN_EXE_ess")).expect("the ess binary is readable"))
}

fn statuses(report: &str, suite: &str) -> BTreeMap<String, &'static str> {
    CountReport::from_json(report, &AdmittedSuite::from_json(suite).unwrap())
        .expect("report/2 of this suite")
        .statuses()
}

fn with_status(report: &str, suite: &str, status: &str) -> Vec<String> {
    statuses(report, suite)
        .into_iter()
        .filter(|(_, it)| *it == status)
        .map(|(id, _)| id)
        .collect()
}

fn label(report: &str) -> String {
    serde_json::from_str::<Value>(report).unwrap()["implementation"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// A declaration of `failures` against the exact suite text `suite`.
fn declaration(suite: &str, implementation: &str, build: &str, failures: &[String]) -> String {
    let admitted = AdmittedSuite::from_json(suite).unwrap();
    let mut failures = failures.to_vec();
    failures.sort();
    let value = json!({
        "format": "ess-known-failures/1",
        "spec_digest": admitted.suite().provenance.spec_digest.to_string(),
        "suite_digest": admitted.digest(),
        "implementation": implementation,
        "implementation_build": build,
        "failures": failures.iter().map(|id| json!({
            "scenario": id,
            "reason": "a tracked defect",
            "tracking": "ORDERS-412",
        })).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&value).unwrap() + "\n"
}

// ---- native `run` --------------------------------------------------------------------------------

/// Billing's synthesized suite with one expectation the billing reference does not meet: the
/// outcome `accepted` of `CreateInvoice` is expected to be `rejected`. A real defect report, from
/// the native runner, against a correct target and a suite asking for something else.
fn failing_suite(directory: &Path) -> PathBuf {
    let synthesized = directory.join("synthesized.json");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        "examples/billing",
        "--out",
        s(&synthesized),
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let mut suite: Value = serde_json::from_str(&read(&synthesized)).unwrap();
    let id = "billing.invoice.CreateInvoice/outcome/accepted";
    let mut changed = 0;
    for step in suite["scenarios"][id]["steps"].as_array_mut().unwrap() {
        if step["step"] == "expect_outcome" && changed == 0 {
            step["outcome"]["outcome"] = json!("rejected");
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "the first expectation is changed");
    let path = directory.join("suite.json");
    std::fs::write(&path, serde_json::to_string_pretty(&suite).unwrap() + "\n").unwrap();
    AdmittedSuite::from_json(&read(&path)).expect("still an admitted suite");
    path
}

fn run(suite: &Path, report: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "verify",
        "conform",
        "run",
        "--target",
        "billing",
        "--report-format",
        "2",
        "--suite",
        s(suite),
        "--report-out",
        s(report),
    ];
    args.extend_from_slice(extra);
    ess(&args)
}

fn without_clock(report: &str) -> Value {
    let mut value: Value = serde_json::from_str(report).unwrap();
    value.as_object_mut().unwrap().remove("completed_at");
    value
}

#[test]
fn run_accounts_known_failures_and_keeps_its_report_verdict_and_exit() {
    let directory = scratch("run");
    let suite = failing_suite(&directory);
    let plain = directory.join("plain.json");
    let unaccounted = run(&suite, &plain, &[]);
    assert_eq!(unaccounted.status.code(), Some(1), "{}", text(&unaccounted));
    let plain_text = read(&plain);
    let suite_text = read(&suite);
    let failed = with_status(&plain_text, &suite_text, "failed");
    assert!(!failed.is_empty(), "the changed expectation fails");
    let known = directory.join("known.json");
    std::fs::write(
        &known,
        declaration(&suite_text, &label(&plain_text), &ess_build(), &failed[..1]),
    )
    .unwrap();

    for strict in [false, true] {
        let report = directory.join(format!("report-{strict}.json"));
        let accounting = directory.join(format!("accounting-{strict}.json"));
        let mut extra = vec![
            "--known-failing",
            s(&known),
            "--accounting-out",
            s(&accounting),
        ];
        if strict {
            extra.push("--strict");
        }
        let output = run(&suite, &report, &extra);
        let baseline = run(
            &suite,
            &directory.join(format!("baseline-{strict}.json")),
            if strict { &["--strict"] } else { &[] },
        );
        assert_eq!(
            output.status.code(),
            baseline.status.code(),
            "the exit status is the run's own: {}",
            text(&output)
        );
        assert_eq!(output.status.code(), Some(1));
        let report_text = read(&report);
        assert_eq!(
            without_clock(&report_text),
            without_clock(&plain_text),
            "the report is unchanged"
        );
        assert_eq!(
            serde_json::from_str::<Value>(&report_text).unwrap()["conformance_status"],
            "failed"
        );
        let written = Accounting::from_json(&read(&accounting)).unwrap();
        written
            .validate(
                &report_text,
                &AdmittedSuite::from_json(&suite_text).unwrap(),
                &read(&known),
            )
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        assert_eq!(written.implementation_build, ess_build());
        assert_eq!(
            written
                .known_failed
                .iter()
                .map(|it| it.scenario.clone())
                .collect::<Vec<_>>(),
            failed[..1].to_vec()
        );
        assert_eq!(written.unexpected_failed, failed[1..].to_vec());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("known failure"),
            "{}",
            text(&output)
        );
    }
}

#[test]
fn run_refuses_a_declaration_or_destination_that_does_not_bind_it_and_writes_nothing() {
    let directory = scratch("run-refused");
    let suite = failing_suite(&directory);
    let plain = directory.join("plain.json");
    run(&suite, &plain, &[]);
    let plain_text = read(&plain);
    let suite_text = read(&suite);
    let failed = with_status(&plain_text, &suite_text, "failed");
    let passing = with_status(&plain_text, &suite_text, "passed");
    let implementation = label(&plain_text);
    let mut stale = failed.clone();
    stale.push(passing[0].clone());
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "another build",
            declaration(&suite_text, &implementation, &host_build(), &failed),
            "known-failures.identity",
        ),
        (
            "a passing scenario",
            declaration(&suite_text, &implementation, &ess_build(), &stale),
            "known-failures.stale",
        ),
        (
            "another implementation",
            declaration(&suite_text, "billing-reference 0.0", &ess_build(), &failed),
            "known-failures.identity",
        ),
        (
            "not a declaration",
            "{}".to_owned(),
            "known-failures.malformed",
        ),
    ];
    for (case, document, code) in cases {
        let known = directory.join(format!("{case}.json"));
        std::fs::write(&known, document).unwrap();
        let report = directory.join(format!("{case}-report.json"));
        let accounting = directory.join(format!("{case}-accounting.json"));
        let output = run(
            &suite,
            &report,
            &[
                "--known-failing",
                s(&known),
                "--accounting-out",
                s(&accounting),
            ],
        );
        assert_eq!(output.status.code(), Some(2), "{case}: {}", text(&output));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "{case}: {}",
            text(&output)
        );
        assert!(
            !report.exists() && !accounting.exists(),
            "{case}: nothing is written"
        );
    }
}

#[test]
fn run_refuses_an_accounting_destination_that_exists_or_is_its_report_and_a_report_one_run() {
    let directory = scratch("run-destination");
    let suite = failing_suite(&directory);
    let plain = directory.join("plain.json");
    run(&suite, &plain, &[]);
    let plain_text = read(&plain);
    let suite_text = read(&suite);
    let failed = with_status(&plain_text, &suite_text, "failed");
    let implementation = label(&plain_text);
    let known = directory.join("good.json");
    std::fs::write(
        &known,
        declaration(&suite_text, &implementation, &ess_build(), &failed),
    )
    .unwrap();
    let report = directory.join("same-report.json");
    let output = run(
        &suite,
        &report,
        &["--known-failing", s(&known), "--accounting-out", s(&report)],
    );
    assert_eq!(output.status.code(), Some(2), "{}", text(&output));
    let existing = directory.join("existing.json");
    std::fs::write(&existing, "kept\n").unwrap();
    let output = run(
        &suite,
        &directory.join("fresh-report.json"),
        &[
            "--known-failing",
            s(&known),
            "--accounting-out",
            s(&existing),
        ],
    );
    assert_eq!(output.status.code(), Some(2), "{}", text(&output));
    assert_eq!(
        read(&existing),
        "kept\n",
        "an existing file is never replaced"
    );
    let output = ess(&[
        "verify",
        "conform",
        "run",
        "--target",
        "billing",
        "--suite",
        s(&suite),
        "--report-out",
        s(&directory.join("one.json")),
        "--known-failing",
        s(&known),
        "--accounting-out",
        s(&directory.join("one-accounting.json")),
    ]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "report/1 has no exact suite binding: {}",
        text(&output)
    );
}

// ---- `report --results` ----------------------------------------------------------------------------

fn results_from(report: &Value) -> Value {
    let mut results = Vec::new();
    for status in ["passed", "failed", "error", "unsupported"] {
        for id in report["outcomes"][status].as_array().unwrap() {
            results.push(json!({"scenario_id": id, "status": status}));
        }
    }
    json!({"format":"ess-conformance-results/1","completed_at":1_700_000_000_000_u64,"results":results})
}

#[test]
fn report_results_writes_the_unchanged_report_its_context_and_the_accounting() {
    let directory = scratch("results");
    let suite = failing_suite(&directory);
    let plain = directory.join("plain.json");
    run(&suite, &plain, &[]);
    let native: Value = serde_json::from_str(&read(&plain)).unwrap();
    let results = directory.join("results.json");
    std::fs::write(&results, results_from(&native).to_string()).unwrap();
    let suite_text = read(&suite);
    let failed = with_status(&read(&plain), &suite_text, "failed");
    let known = directory.join("known.json");
    std::fs::write(
        &known,
        declaration(&suite_text, "downstream-impl 2.0", &host_build(), &failed),
    )
    .unwrap();
    let report = |name: &str, extra: &[&str]| {
        let mut args = vec![
            "verify",
            "conform",
            "report",
            "--suite",
            s(&suite),
            "--results",
            s(&results),
            "--implementation",
            "downstream-impl 2.0",
        ];
        let out = directory.join(name);
        args.extend_from_slice(&["--report-out", s(&out)]);
        args.extend_from_slice(extra);
        (ess(&args), out)
    };
    let (ordinary, ordinary_out) = report("ordinary.json", &[]);
    assert_eq!(ordinary.status.code(), Some(0), "{}", text(&ordinary));
    let context = directory.join("execution.json");
    let accounting = directory.join("accounting.json");
    let build = host_build();
    let (accounted, accounted_out) = report(
        "accounted.json",
        &[
            "--known-failing",
            s(&known),
            "--accounting-out",
            s(&accounting),
            "--implementation-build",
            &build,
            "--execution-context-out",
            s(&context),
        ],
    );
    assert_eq!(accounted.status.code(), Some(0), "{}", text(&accounted));
    let report_text = read(&accounted_out);
    assert_eq!(
        report_text,
        read(&ordinary_out),
        "the external report is unchanged"
    );
    let admitted = AdmittedSuite::from_json(&suite_text).unwrap();
    let written = ExecutionContext::from_json(&read(&context)).unwrap();
    written.admit(&report_text, &admitted).unwrap();
    assert_eq!(written.implementation_build(), build);
    Accounting::from_json(&read(&accounting))
        .unwrap()
        .validate(&report_text, &admitted, &read(&known))
        .unwrap_or_else(|refusal| panic!("{refusal}"));

    // The build is the caller's to state: without it, nothing is written.
    let (missing, missing_out) = report(
        "missing.json",
        &[
            "--known-failing",
            s(&known),
            "--accounting-out",
            s(&directory.join("a2.json")),
        ],
    );
    assert_eq!(missing.status.code(), Some(2), "{}", text(&missing));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("--implementation-build"));
    assert!(!missing_out.exists());
    let (invalid, invalid_out) = report(
        "invalid.json",
        &[
            "--known-failing",
            s(&known),
            "--accounting-out",
            s(&directory.join("a3.json")),
            "--implementation-build",
            "v2.0",
            "--execution-context-out",
            s(&directory.join("c3.json")),
        ],
    );
    assert_eq!(invalid.status.code(), Some(2), "{}", text(&invalid));
    assert!(!invalid_out.exists() && !directory.join("a3.json").exists());
}

// ---- generated runners ------------------------------------------------------------------------------

/// The billing Go package `synthesize --target go` emits, with the hand-written billing target.
fn go_billing(directory: &Path) -> PathBuf {
    let module = directory.join("go-billing");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        "examples/billing",
        "--target",
        "go",
        "--out",
        s(&module),
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-billing");
    for file in ["target.go", "target_test.go"] {
        std::fs::copy(fixture.join(file), module.join(file)).unwrap();
    }
    std::fs::write(module.join("go.mod"), "module essbilling\n\ngo 1.24\n").unwrap();
    module
}

/// Runs the Go billing package over `suite` with its `reversed-order` defect, writing report/2 and
/// the host's execution context into `out`; returns the `go test` output.
fn go_run(module: &Path, suite: &str, out: &Path, strict: bool) -> Output {
    std::fs::write(module.join("essconform/suite.json"), suite).unwrap();
    let mut command = Command::new("go");
    command
        .args(["test", "-count=1", "-run", "^TestConformance$", "./..."])
        .current_dir(module)
        .env("GOWORK", "off")
        .env("ESS_BREAK", "reversed-order")
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", out.join("report.json"))
        .env("ESS_IMPLEMENTATION_BUILD", host_build())
        .env("ESS_EXECUTION_CONTEXT_OUT", out.join("execution.json"))
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env_remove("ESS_CONFORMANCE_STRICT");
    if strict {
        command.env("ESS_CONFORMANCE_STRICT", "1");
    }
    command.output().expect("go test runs")
}

#[test]
fn a_generated_go_report_is_accounted_from_its_own_bytes_and_context() {
    if !tool("go") {
        return;
    }
    let directory = scratch("go-report");
    let module = go_billing(&directory);
    let suite = module.join("essconform/suite.json");
    let suite_text = read(&suite);
    let out = directory.join("run");
    std::fs::create_dir_all(&out).unwrap();
    let ran = go_run(&module, &suite_text, &out, false);
    assert!(
        !ran.status.success(),
        "the defective target fails: {}",
        text(&ran)
    );
    let report = out.join("report.json");
    let context = out.join("execution.json");
    let report_text = read(&report);
    let failed = with_status(&report_text, &suite_text, "failed");
    assert!(
        !failed.is_empty(),
        "reversed-order fails the ordering scenarios"
    );
    let known = directory.join("known.json");
    std::fs::write(
        &known,
        declaration(&suite_text, &label(&report_text), &host_build(), &failed),
    )
    .unwrap();

    let accounting = directory.join("accounting.json");
    let paths = (suite.as_path(), report.as_path(), context.as_path());
    let output = observe(paths, &known, &accounting, &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output));
    assert_eq!(
        read(&report),
        report_text,
        "the observed report is never rewritten"
    );
    let written = Accounting::from_json(&read(&accounting)).unwrap();
    written
        .validate(
            &report_text,
            &AdmittedSuite::from_json(&suite_text).unwrap(),
            &read(&known),
        )
        .unwrap();
    assert_eq!(written.unexpected_failed, Vec::<String>::new());
    assert_eq!(written.implementation_build, host_build());
    let raw: Value = serde_json::from_str(&report_text).unwrap();
    assert_eq!(raw["producer_profile"], "go-scenario-status/2");
    assert_eq!(
        raw["conformance_status"], "failed",
        "the report still fails"
    );

    // Caller provenance cannot replace what the runner's host observed.
    for (case, extra) in [
        ("results", vec!["--results", s(&report)]),
        ("implementation", vec!["--implementation", "someone 1"]),
        ("build", vec!["--implementation-build", "sha256:0"]),
        ("report-out", vec!["--report-out", "x.json"]),
    ] {
        let output = observe(
            paths,
            &known,
            &directory.join(format!("{case}-acc.json")),
            &extra,
        );
        assert_eq!(output.status.code(), Some(2), "{case}: {}", text(&output));
        assert!(!directory.join(format!("{case}-acc.json")).exists());
    }
    // Another build, or other report bytes, is refused and writes nothing.
    let other = directory.join("other-build.json");
    std::fs::write(
        &other,
        declaration(
            &suite_text,
            &label(&report_text),
            &format!("sha256:{}", "e".repeat(64)),
            &failed,
        ),
    )
    .unwrap();
    let output = observe(paths, &other, &directory.join("other-acc.json"), &[]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output));
    let tampered = directory.join("tampered.json");
    std::fs::write(&tampered, report_text.replace('\n', "\r\n")).unwrap();
    let output = observe(
        (&suite, &tampered, &context),
        &known,
        &directory.join("tampered-acc.json"),
        &[],
    );
    assert_eq!(output.status.code(), Some(2), "{}", text(&output));
    assert!(!directory.join("tampered-acc.json").exists());

    // Strict execution still fails, and still writes its report and context.
    strict_go_run_still_fails_and_writes(&module, &suite_text, &directory.join("strict"));
}

/// `ess verify conform report --observed-report`, over `(suite, report, context)`.
fn observe(
    (suite, report, context): (&Path, &Path, &Path),
    known: &Path,
    accounting: &Path,
    extra: &[&str],
) -> Output {
    let mut args = vec![
        "verify",
        "conform",
        "report",
        "--suite",
        s(suite),
        "--observed-report",
        s(report),
        "--execution-context",
        s(context),
        "--known-failing",
        s(known),
        "--accounting-out",
        s(accounting),
    ];
    args.extend_from_slice(extra);
    ess(&args)
}

fn strict_go_run_still_fails_and_writes(module: &Path, suite_text: &str, out: &Path) {
    std::fs::create_dir_all(out).unwrap();
    let strict = go_run(module, suite_text, out, true);
    assert!(!strict.status.success(), "{}", text(&strict));
    ExecutionContext::from_json(&read(&out.join("execution.json")))
        .unwrap()
        .admit(
            &read(&out.join("report.json")),
            &AdmittedSuite::from_json(suite_text).unwrap(),
        )
        .unwrap();
}

/// A small suite: one scenario passes, two fail, one needs a command the target cannot expose.
fn review_suite() -> ConformanceSuite {
    let execute = |command: &str| json!({"step": "execute_command", "command": command});
    let expect = json!({"step": "expect_outcome", "outcome": {"command": "review.count.Do", "outcome": "done"}});
    let document = json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "review",
            "specification_version": "v1", "spec_digest": "a".repeat(64),
            "contract_digest": "a".repeat(64)},
        "scenarios": {
            "review.count/authored/passes": {"purpose": "Passes", "steps": [], "source": []},
            "review.count/authored/known": {"purpose": "Fails, as declared",
                "steps": [execute("review.count.Do"), expect.clone()], "source": []},
            "review.count/authored/unexpected": {"purpose": "Fails, undeclared",
                "steps": [execute("review.count.Do"), expect], "source": []},
            "review.count/authored/unsupported": {"purpose": "Cannot be answered",
                "steps": [execute("review.count.Missing")], "source": []},
        }
    });
    ConformanceSuite::from_json(&document.to_string()).expect("a suite")
}

const REVIEW_GO: &str = r#"package essconform

import "testing"

type knownTarget struct{ Target }

func (knownTarget) Identity() (Identity, error) { return Identity{Name: "known-review", Version: "1"}, nil }
func (knownTarget) BeginScenario(ScenarioContext) error { return nil }
func (knownTarget) EndScenario(ScenarioContext) error   { return nil }
func (knownTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command == "review.count.Missing" {
		return CommandResult{}, ErrUnsupported
	}
	return CommandResult{Outcome: "other"}, nil
}
func TestKnown(t *testing.T) { Run(t, func() Target { return knownTarget{} }) }
"#;

const REVIEW_TS_TARGET: &str = r"import { ErrUnsupported } from './dist/runtime.js';
export function makeTarget() {
  return {
    identity: () => ({ name: 'known-review', version: '1' }),
    beginScenario() {},
    endScenario() {},
    executeCommand({ command }) {
      if (command === 'review.count.Missing') throw ErrUnsupported;
      return { outcome: 'other' };
    },
  };
}
";

const REVIEW_TS_DRIVER: &str = r"import test from 'node:test';
import { run } from './dist/runtime.js';
import { makeTarget } from './target.mjs';
await test('suite', (t) => run(t, makeTarget));
";

/// The Rust runner's account of the same target.
struct KnownReview;

impl ConformanceTarget for KnownReview {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("known-review", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == "review.count.Missing" {
            return Err(TargetError::unsupported("execute command", "not exposed"));
        }
        Ok(SemanticCommandResult::undeclared())
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("query view", "not exposed"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcome", "not exposed"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redeliver", "not exposed"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Ok(Vec::new())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

/// Where each review run below keeps the suite it ran, relative to the directory it returns.
const SUITE_BESIDE: &str = "suite.json";

/// Writes `files` under `directory`.
fn write_artifacts(directory: &Path, files: impl IntoIterator<Item = (String, String)>) {
    for (relative, contents) in files {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

/// The generated Go runner over `suite` against the review target, with its host's context; the
/// directory holding its report, context and a copy of the suite it ran.
fn go_review(suite: &ConformanceSuite) -> PathBuf {
    let directory = scratch("review-go");
    write_artifacts(
        &directory,
        ess_conformance::go::emit(suite)
            .unwrap()
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    std::fs::write(directory.join("go.mod"), "module knownreview\n\ngo 1.24\n").unwrap();
    std::fs::write(directory.join("essconform/known_test.go"), REVIEW_GO).unwrap();
    let output = Command::new("go")
        .args(["test", "-count=1", "-run", "^TestKnown$", "./..."])
        .current_dir(&directory)
        .env("GOWORK", "off")
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", directory.join("report.json"))
        .env("ESS_IMPLEMENTATION_BUILD", host_build())
        .env(
            "ESS_EXECUTION_CONTEXT_OUT",
            directory.join("execution.json"),
        )
        .env_remove("ESS_CONFORMANCE_STRICT")
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", text(&output));
    std::fs::copy(
        directory.join("essconform/suite.json"),
        directory.join(SUITE_BESIDE),
    )
    .unwrap();
    directory
}

/// The generated TypeScript runner over `suite` against the review target, as [`go_review`].
fn typescript_review(suite: &ConformanceSuite) -> PathBuf {
    let directory = scratch("review-ts");
    write_artifacts(
        &directory,
        ess_conformance::ts::emit(suite)
            .unwrap()
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    let package = directory.join(ess_conformance::ts::PACKAGE);
    std::fs::write(package.join("target.mjs"), REVIEW_TS_TARGET).unwrap();
    std::fs::write(package.join("driver.mjs"), REVIEW_TS_DRIVER).unwrap();
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", text(&compiled));
    let output = Command::new("node")
        .args(["--test", "driver.mjs"])
        .current_dir(&package)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", package.join("report.json"))
        .env("ESS_IMPLEMENTATION_BUILD", host_build())
        .env("ESS_EXECUTION_CONTEXT_OUT", package.join("execution.json"))
        .env_remove("ESS_CONFORMANCE_STRICT")
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", text(&output));
    package
}

/// Runs the accounting-only report mode over a runner's report and context in `dir`.
fn account_observed(dir: &Path, suite: &Path, known: &Path) -> (Output, PathBuf) {
    let accounting = dir.join(format!(
        "accounting-{}.json",
        known.file_stem().unwrap().to_string_lossy()
    ));
    let output = ess(&[
        "verify",
        "conform",
        "report",
        "--suite",
        s(suite),
        "--observed-report",
        s(&dir.join("report.json")),
        "--execution-context",
        s(&dir.join("execution.json")),
        "--known-failing",
        s(known),
        "--accounting-out",
        s(&accounting),
    ]);
    (output, accounting)
}

#[test]
fn generated_go_and_typescript_failed_and_unsupported_account_as_the_native_run_does() {
    let suite = review_suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let native_run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &KnownReview);
    let native = CountReport::from_run(&native_run, &admitted)
        .unwrap()
        .to_canonical_json()
        .unwrap();
    let native_statuses = statuses(&native, admitted.original_json());
    assert_eq!(
        native_statuses.values().copied().collect::<Vec<_>>(),
        ["failed", "passed", "failed", "unsupported"],
        "known, passes, unexpected, unsupported: {native_statuses:?}"
    );
    let known_id = vec!["review.count/authored/known".to_owned()];
    let native_accounting = {
        let declaration = ess_conformance::known_failures::Declaration::from_json(&declaration(
            admitted.original_json(),
            "known-review 1",
            &host_build(),
            &known_id,
        ))
        .unwrap();
        ess_conformance::known_failures::account(&native, &admitted, &declaration, &host_build())
            .unwrap()
    };
    assert_eq!(
        native_accounting.unexpected_failed,
        ["review.count/authored/unexpected"]
    );

    let mut ran = Vec::new();
    if tool("go") {
        ran.push(("go", go_review(&suite)));
    }
    if tool("tsc") && tool("node") {
        ran.push(("typescript", typescript_review(&suite)));
    }
    let ran = ran
        .into_iter()
        .map(|(runner, dir)| (runner, dir.clone(), dir.join(SUITE_BESIDE)));
    for (runner, dir, suite_path) in ran {
        let suite_text = read(&suite_path);
        let report_text = read(&dir.join("report.json"));
        let raw: Value = serde_json::from_str(&report_text).unwrap();
        assert_eq!(raw["producer_profile"], "go-scenario-status/2", "{runner}");
        assert_eq!(
            statuses(&report_text, &suite_text),
            native_statuses,
            "{runner}: Failed and Unsupported stay what the native run calls them"
        );
        let known = dir.join("known.json");
        std::fs::write(
            &known,
            declaration(&suite_text, "known-review 1", &host_build(), &known_id),
        )
        .unwrap();
        let (output, accounting) = account_observed(&dir, &suite_path, &known);
        assert_eq!(output.status.code(), Some(0), "{runner}: {}", text(&output));
        let written = Accounting::from_json(&read(&accounting)).unwrap();
        assert_eq!(
            written.known_failed, native_accounting.known_failed,
            "{runner}"
        );
        assert_eq!(
            written.unexpected_failed, native_accounting.unexpected_failed,
            "{runner}"
        );
        assert_eq!(written.counts, native_accounting.counts, "{runner}");
        // An unsupported scenario is never a known failure.
        let unsupported = dir.join("unsupported.json");
        std::fs::write(
            &unsupported,
            declaration(
                &suite_text,
                "known-review 1",
                &host_build(),
                &["review.count/authored/unsupported".to_owned()],
            ),
        )
        .unwrap();
        let (output, accounting) = account_observed(&dir, &suite_path, &unsupported);
        assert_eq!(output.status.code(), Some(2), "{runner}: {}", text(&output));
        assert!(String::from_utf8_lossy(&output.stderr).contains("known-failures.not-failed"));
        assert!(!accounting.exists());
    }
}

#[test]
fn a_generated_runner_refuses_an_invalid_build_before_any_target_is_made() {
    if !tool("go") {
        return;
    }
    let directory = scratch("review-go-refused");
    for artifact in ess_conformance::go::emit(&review_suite()).unwrap() {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(directory.join("go.mod"), "module knownreview\n\ngo 1.24\n").unwrap();
    std::fs::write(
        directory.join("essconform/known_test.go"),
        REVIEW_GO.replace(
            "func (knownTarget) Identity() (Identity, error) {",
            "func (knownTarget) Identity() (Identity, error) { panic(\"a target was made\")\n",
        ),
    )
    .unwrap();
    for (case, build, context) in [
        (
            "not a digest",
            "v1.2.3".to_owned(),
            directory.join("c1.json"),
        ),
        (
            "the report path",
            host_build(),
            directory.join("report.json"),
        ),
    ] {
        let output = Command::new("go")
            .args(["test", "-count=1", "-run", "^TestKnown$", "./..."])
            .current_dir(&directory)
            .env("GOWORK", "off")
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", directory.join("report.json"))
            .env("ESS_IMPLEMENTATION_BUILD", &build)
            .env("ESS_EXECUTION_CONTEXT_OUT", &context)
            .output()
            .unwrap();
        let printed = text(&output);
        assert!(!output.status.success(), "{case}: {printed}");
        assert!(
            printed.contains("execution context configuration"),
            "{case}: {printed}"
        );
        assert!(!printed.contains("a target was made"), "{case}: {printed}");
        assert!(!directory.join("report.json").exists(), "{case}");
    }
}

// ---- `mutate` -----------------------------------------------------------------------------------------

fn mutate(args: &[&str]) -> Output {
    let mut all = vec!["verify", "conform", "mutate"];
    all.extend_from_slice(args);
    ess(&all)
}

/// Runs the Go billing package over every suite of the emission at `emitted`.
fn go_over_emission(module: &Path, emitted: &Path) {
    let manifest: Value = serde_json::from_str(&read(&emitted.join("manifest.json"))).unwrap();
    let mut dirs = vec![manifest["baseline"]["dir"].as_str().unwrap().to_owned()];
    dirs.extend(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|it| it["dir"].as_str().map(str::to_owned)),
    );
    for dir in dirs {
        let at = emitted.join(&dir);
        let output = go_run(module, &read(&at.join("suite.json")), &at, false);
        assert!(at.join("report.json").exists(), "{dir}: {}", text(&output));
    }
}

#[test]
fn mutation_audit_of_a_generated_go_runner_under_a_bound_declaration() {
    if !tool("go") {
        return;
    }
    let directory = scratch("mutate-go");
    let module = go_billing(&directory);
    let first = directory.join("first");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--emit",
        s(&first),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output));
    go_over_emission(&module, &first);
    // Without a declaration the defective baseline is refused, as it always was.
    let refused = mutate(&["--collect", s(&first)]);
    assert_eq!(refused.status.code(), Some(3), "{}", text(&refused));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("ESS-MUTATE-001"));

    let suite_text = read(&first.join("baseline/suite.json"));
    let report_text = read(&first.join("baseline/report.json"));
    let failed = with_status(&report_text, &suite_text, "failed");
    let known = directory.join("known.json");
    let declared = declaration(&suite_text, &label(&report_text), &host_build(), &failed);
    std::fs::write(&known, &declared).unwrap();
    // A declaration cannot be bound to an emission after its suites ran.
    let late = mutate(&["--collect", s(&first), "--known-failing", s(&known)]);
    assert_eq!(late.status.code(), Some(2), "{}", text(&late));
    assert!(String::from_utf8_lossy(&late.stderr).contains("known-failures.unbound"));

    let bound = directory.join("bound");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--emit",
        s(&bound),
        "--known-failing",
        s(&known),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output));
    assert_eq!(
        read(&bound.join("known-failures.json")),
        declared,
        "copied byte for byte"
    );
    let manifest: Value = serde_json::from_str(&read(&bound.join("manifest.json"))).unwrap();
    assert_eq!(manifest["format"], "ess-mutation-manifest/4");
    assert_eq!(
        manifest["known_failures"]["implementation_build"],
        host_build()
    );
    go_over_emission(&module, &bound);
    let report_out = directory.join("mutation.json");
    let collected = mutate(&[
        "--collect",
        s(&bound),
        "--format",
        "json",
        "--report-out",
        s(&report_out),
    ]);
    let report: Value = serde_json::from_str(&read(&report_out)).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/4");
    let declared_ids: Vec<&str> = report["known_failures"]["failures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["scenario"].as_str().unwrap())
        .collect();
    assert_eq!(
        declared_ids,
        failed.iter().map(String::as_str).collect::<Vec<_>>()
    );
    let counts = &report["counts"];
    assert!(counts["killed"].as_u64().unwrap() > 0, "{report:#}");
    for entry in report["mutants"].as_array().unwrap() {
        for killer in entry["killers"].as_array().into_iter().flatten() {
            assert!(!failed.iter().any(|it| it == killer), "{entry:#}");
        }
    }
    let expected_exit = mutation_exit(counts);
    assert_eq!(
        collected.status.code(),
        Some(expected_exit),
        "{}",
        text(&collected)
    );
    let shown = mutate(&["--collect", s(&bound)]);
    let printed = String::from_utf8_lossy(&shown.stdout).to_string();
    assert!(
        printed
            .lines()
            .nth(1)
            .is_some_and(|line| line.starts_with("known failure ")),
        "known failures lead the text:\n{printed}"
    );
    // Only the bound bytes are accepted again.
    only_the_bound_bytes_collect(&bound, (&known, &declared), expected_exit);
}

/// The exit status `mutate --collect` gives a report of `counts`.
fn mutation_exit(counts: &Value) -> i32 {
    if counts["survived"].as_u64() > Some(0) {
        1
    } else if counts["unwitnessed"].as_u64() > Some(0) || counts["inconclusive"].as_u64() > Some(0)
    {
        3
    } else {
        0
    }
}

/// `--collect` under the bound bytes `declared` (at `known`) again, and under other bytes.
fn only_the_bound_bytes_collect(
    bound: &Path,
    (known, declared): (&Path, &str),
    expected_exit: i32,
) {
    let changed = bound.with_file_name("changed.json");
    std::fs::write(&changed, declared.replace("ORDERS-412", "ORDERS-413")).unwrap();
    let output = mutate(&["--collect", s(bound), "--known-failing", s(&changed)]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output));
    let again = mutate(&["--collect", s(bound), "--known-failing", s(known)]);
    assert_eq!(again.status.code(), Some(expected_exit), "{}", text(&again));
}

#[test]
fn the_built_in_audit_binds_the_executable_build_and_refuses_a_stale_declaration() {
    let directory = scratch("mutate-target");
    let emitted = directory.join("emitted");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--emit",
        s(&emitted),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output));
    let suite_text = read(&emitted.join("baseline/suite.json"));
    let admitted = AdmittedSuite::from_json(&suite_text).unwrap();
    let executed = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &ess_conformance::reference::Billing::new());
    let implementation = executed.implementation.to_string();
    let any = admitted
        .suite()
        .scenarios
        .keys()
        .next()
        .unwrap()
        .to_string();
    for (case, build, code) in [
        ("another build", host_build(), "known-failures.identity"),
        (
            "this build, a passing scenario",
            ess_build(),
            "known-failures.stale",
        ),
    ] {
        let known = directory.join(format!("{case}.json"));
        std::fs::write(
            &known,
            declaration(
                &suite_text,
                &implementation,
                &build,
                std::slice::from_ref(&any),
            ),
        )
        .unwrap();
        let output = mutate(&[
            "--path",
            "examples/billing",
            "--class",
            "error-swap",
            "--target",
            "billing",
            "--known-failing",
            s(&known),
        ]);
        assert_eq!(output.status.code(), Some(2), "{case}: {}", text(&output));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "{case}: {}",
            text(&output)
        );
    }
}
