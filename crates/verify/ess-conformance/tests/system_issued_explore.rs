//! The generated explorers never send a command the system issues itself (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Target interface*): it is no client's operation, so
//! each explorer leaves it out and says why, in Go and in TypeScript.
//!
//! The model is `tests/fixtures/system-issued-queue.yaml`; the target answers every question
//! unsupported, so what is left out for the system's step is the plan's decision, not the target's.

mod support_go;

use std::path::Path;
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const MODEL: &str = include_str!("fixtures/system-issued-queue.yaml");
const ASSIGN: &str = "queueing.entry.Assign";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("system-issued-queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.select_fresh_format();
    suite
}

/// The one exclusion naming the system's step, from an explorer's `excluded` list.
fn excluded_assign(excluded: &Value, log: &str) -> String {
    let reasons: Vec<&str> = excluded
        .as_array()
        .unwrap_or_else(|| panic!("an exclusion list: {excluded}\n{log}"))
        .iter()
        .filter(|exclusion| exclusion["subject"] == ASSIGN)
        .map(|exclusion| exclusion["reason"].as_str().unwrap())
        .collect();
    assert_eq!(reasons.len(), 1, "{excluded}\n{log}");
    reasons[0].to_owned()
}

const GO_DRIVER: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

type stubTarget struct{}

func (stubTarget) Identity() (Identity, error)         { return Identity{Name: "stub", Version: "1"}, nil }
func (stubTarget) BeginScenario(ScenarioContext) error { return nil }
func (stubTarget) EndScenario(ScenarioContext) error   { return nil }
func (stubTarget) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, ErrUnsupported
}
func (stubTarget) QueryView(ViewRequest) (ViewResult, error) { return ViewResult{}, ErrUnsupported }
func (stubTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (stubTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (stubTarget) RedeliverEvent(RedeliveryRequest) error               { return ErrUnsupported }
func (stubTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestExcluded(t *testing.T) {
	result, err := Explore(func() Target { return stubTarget{} }, ExploreOptions{Seeds: 1, Steps: 1})
	if err != nil {
		t.Fatal(err)
	}
	raw, _ := json.Marshal(result.Excluded)
	if err := os.WriteFile(os.Getenv("ESS_EXCLUDED"), raw, 0o644); err != nil {
		t.Fatal(err)
	}
}
"#;

#[test]
fn the_go_explorer_leaves_out_a_system_issued_command() {
    let ir = ir();
    let suite = suite(&ir);
    let directory = support_go::directory("explore-system-issued");
    for artifact in ess_conformance::go::emit_with_model(&suite, &ir).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/goparity\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform").join("excluded_test.go"),
        GO_DRIVER,
    )
    .unwrap();
    let output = directory.join("excluded.json");
    let run = support_go::go_test(
        &directory,
        "TestExcluded",
        &[("ESS_EXCLUDED", output.to_str().unwrap())],
    );
    assert!(run.success, "{}", run.log);
    let excluded: Value = serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    let reason = excluded_assign(&excluded, &run.log);
    assert!(reason.contains("the system issues it itself"), "{reason}");
}

const NODE_DRIVER: &str = r"import {writeFileSync} from 'node:fs';
import {explore, unsupported} from './dist/index.js';
const refuse = async () => { throw unsupported('stub'); };
const target = () => ({
  identity: async () => ({name: 'stub', version: '1'}),
  beginScenario: async () => {},
  endScenario: async () => {},
  executeCommand: refuse,
  queryView: refuse,
  observeEvents: refuse,
  configureExternalOutcome: refuse,
  redeliverEvent: refuse,
  observeInvocations: refuse,
});
const result = await explore(target, {seeds: 1, steps: 1});
writeFileSync(process.argv[2], JSON.stringify(result.excluded));
";

#[test]
fn the_typescript_explorer_leaves_out_a_system_issued_command() {
    let ir = ir();
    let suite = suite(&ir);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("explore-system-issued-ts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit_with_model(&suite, &ir).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = root.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let compiled = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(compiled.status.success(), "{compiled:?}");
    std::fs::write(package.join("driver.mjs"), NODE_DRIVER).unwrap();
    let output = package.join("excluded.json");
    let run = Command::new("node")
        .arg(package.join("driver.mjs"))
        .arg(&output)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(run.status.success(), "{log}");
    let excluded: Value = serde_json::from_str(
        &std::fs::read_to_string(&output).unwrap_or_else(|e| panic!("{e}: {log}")),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&root);
    let reason = excluded_assign(&excluded, &log);
    assert!(reason.contains("the system issues it itself"), "{reason}");
}
