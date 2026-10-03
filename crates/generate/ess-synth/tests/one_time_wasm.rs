//! The real native observer and shared target execute inside WebAssembly, through browser glue.
//! This deliberately does not claim the fixed billing lab accepts arbitrary conformance suites.
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const HOST: &str = r#"
use std::cell::RefCell;
use ess_conformance::{AdmittedSuite, Runner};
use serde_json::{json, Value};
use support::{Mode, Service, FIRST};
use support as support_one_time;
use resources::{ResourceMode, ResourceService};
use fields::{FieldMode, FieldService};

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}

fn run(request: &str) -> String {
    let request: Value = serde_json::from_str(request).unwrap();
    if let Some(input) = request.get("input") {
        let input = ess_conformance::coverage::AdmittedInput::from_json(input.as_str().unwrap()).unwrap();
        let mode: Mode = serde_json::from_value(request["mode"].clone()).unwrap();
        let target = Service::new(mode);
        return execute(input.selected(), &target, || (target.trace(), target.returned_plaintexts()));
    }
    let suite = AdmittedSuite::from_json(request["suite"].as_str().unwrap()).unwrap();
    if let Some(field) = request.get("field") {
        let mode = FieldMode::ALL.into_iter()
            .find(|mode| serde_json::to_value(mode).unwrap() == *field).unwrap();
        let target = FieldService::new(mode);
        return execute(&suite, &target, || (target.inner.trace(), target.returned_plaintexts()));
    }
    if let Some(resource) = request.get("resource") {
        let mode = ResourceMode::ALL.into_iter()
            .find(|mode| serde_json::to_value(mode).unwrap() == *resource).unwrap();
        let target = ResourceService::new(mode);
        return execute(&suite, &target, || (target.inner.trace(), target.inner.returned_plaintexts()));
    }
    let mode: Mode = serde_json::from_value(request["mode"].clone()).unwrap();
    let target = Service::new(mode);
    execute(&suite, &target, || (target.trace(), target.returned_plaintexts()))
}

fn execute<T: ess_conformance::target::ConformanceTarget>(
    suite: &AdmittedSuite, target: &T,
    observations: impl FnOnce() -> (Vec<&'static str>, Vec<String>),
) -> String {
    let report = Runner::for_suite(suite.suite()).run_admitted(suite, target);
    let counts = ess_conformance::counts::CountReport::from_run(&report, suite).unwrap();
    let (trace, plaintexts) = observations();
    let diagnostics = serde_json::to_string(&report.scenarios).unwrap();
    let count_bytes = counts.to_canonical_json().unwrap();
    let mut all_counts = serde_json::to_value(counts.counts()).unwrap();
    let total = all_counts.as_object_mut().unwrap().remove("total").unwrap();
    let answer = json!({
        "status": report.scenarios[0].status,
        "scenario_ids": report.scenarios.iter().map(|s| &s.scenario).collect::<Vec<_>>(),
        "counts": all_counts, "total": total, "diagnostics": diagnostics,
        "callback_trace": trace, "count_report": count_bytes, "redacted": true,
    }).to_string();
    // Captures stay inside WASM. Even the test result carries no captured plaintext.
    for value in plaintexts.into_iter().chain([FIRST.to_owned()]) {
        if answer.contains(&value) {
            return json!({"redacted": false}).to_string();
        }
    }
    answer
}

#[no_mangle]
pub extern "C" fn ess_input_reserve(length: u32) -> u32 {
    INPUT.with(|held| {
        let mut held = held.borrow_mut();
        *held = vec![0; length as usize];
        held.as_mut_ptr() as u32
    })
}
#[no_mangle]
pub extern "C" fn ess_dispatch() -> u32 {
    let answer = INPUT.with(|held| run(std::str::from_utf8(&held.borrow()).unwrap()));
    OUTPUT.with(|held| {
        *held.borrow_mut() = answer;
        held.borrow().as_ptr() as u32
    })
}
#[no_mangle]
pub extern "C" fn ess_output_len() -> u32 {
    OUTPUT.with(|held| held.borrow().len() as u32)
}
"#;

const DRIVER: &str = r"
import {readFileSync} from 'node:fs';
import {join, dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
const [glue, wasm, fixtures] = process.argv.slice(2);
const {open} = await import(pathToFileURL(glue));
const system = await open(readFileSync(wasm));
const manifest = JSON.parse(readFileSync(join(fixtures, 'manifest.json'), 'utf8'));
const answers = manifest.map(item => ({
  case: item.case,
  answer: system.request({mode: item.case, suite: readFileSync(join(fixtures, item.suite), 'utf8')}),
}));
const resources = JSON.parse(readFileSync(join(dirname(fixtures), 'one-time-resources.json'), 'utf8'));
const resourceAnswers = resources.map(item => ({
  case: item.case,
  answer: system.request({resource: item.case, suite: readFileSync(join(fixtures, 'view.json'), 'utf8')}),
}));
const fieldRoot = join(dirname(fixtures), 'one-time-fields');
const fieldCases = JSON.parse(readFileSync(join(fieldRoot, 'manifest.json'), 'utf8'));
const fieldAnswers = fieldCases.map(item => ({
  case: item.case,
  answer: system.request({field: item.case, suite: readFileSync(join(fieldRoot, 'suite.json'), 'utf8')}),
}));
const coverageRoot = join(dirname(fixtures), 'one-time-coverage');
const coverageCases = JSON.parse(readFileSync(join(coverageRoot, 'manifest.json'), 'utf8'));
const coverageAnswers = coverageCases.map(item => ({
  case: item.case,
  answer: system.request({mode: item.case, input: readFileSync(join(coverageRoot, 'input.json'), 'utf8')}),
}));
const windowRoot = join(dirname(fixtures), 'one-time-windows');
const windowCases = JSON.parse(readFileSync(join(windowRoot, 'manifest.json'), 'utf8'));
const windowAnswers = windowCases.map(item => ({
  case: item.case,
  answer: system.request({mode: item.case, suite: readFileSync(join(windowRoot, 'suite.json'), 'utf8')}),
}));
process.stdout.write(JSON.stringify({execution: answers, resources: resourceAnswers, fields: fieldAnswers, coverage: coverageAnswers, windows: windowAnswers}));
";

fn run(command: &mut Command, label: &str) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{label}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn build_host(root: &Path, conformance: &Path) -> PathBuf {
    std::fs::create_dir_all(root.join("src")).unwrap();
    let primitives = conformance
        .join("../../specify/ess-primitives")
        .canonicalize()
        .unwrap();
    let conformance_path = serde_json::to_string(conformance.to_str().unwrap()).unwrap();
    let primitives_path = serde_json::to_string(primitives.to_str().unwrap()).unwrap();
    let manifest = format!(
        "[package]\nname = \"one-time-wasm-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
         [workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\n\
         ess-conformance = {{path = {conformance_path}}}\n\
         ess-primitives = {{path = {primitives_path}}}\n\
         serde = {{version = \"1\", features = [\"derive\"]}}\n\
         serde_json = {{version = \"1\", features = [\"arbitrary_precision\"]}}\n"
    );
    std::fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let support = conformance.join("tests/support_one_time/mod.rs");
    let support_path = serde_json::to_string(support.to_str().unwrap()).unwrap();
    let resources = conformance.join("tests/support_one_time/resources.rs");
    let resources_path = serde_json::to_string(resources.to_str().unwrap()).unwrap();
    let fields = conformance.join("tests/support_one_time/fields.rs");
    let fields_path = serde_json::to_string(fields.to_str().unwrap()).unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        format!("#[allow(dead_code)]\n#[path = {support_path}]\nmod support;\n#[allow(dead_code)]\n#[path = {resources_path}]\nmod resources;\n#[allow(dead_code)]\n#[path = {fields_path}]\nmod fields;\n{HOST}"),
    )
    .unwrap();
    let target = root.join("target");
    let cargo = std::env::var_os("CARGO").unwrap();
    run(
        Command::new(&cargo)
            .args(["generate-lockfile", "--offline"])
            .current_dir(root),
        "host lock",
    );
    run(
        Command::new(cargo)
            .args([
                "build",
                "--locked",
                "--offline",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .arg("--target-dir")
            .arg(&target)
            .current_dir(root)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("RUSTFLAGS", "-D warnings"),
        "actual WASM observer build",
    );
    target.join("wasm32-unknown-unknown/debug/one_time_wasm_host.wasm")
}

#[test]
fn shared_disclosure_controls_execute_inside_wasm_through_browser_bridge() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("one-time-wasm");
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-conformance")
        .canonicalize()
        .unwrap();
    let wasm = build_host(&root, &conformance);
    // Use the actual emitted browser transport. This unmarked model only supplies transport;
    // the execution request carries the frozen marked suite, unchanged, into the WASM runner.
    let source = "format: ess/20\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true}]\n";
    let spec = Specification::assemble([(
        Source::new("transport.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let web = synthesize_for(&model, Target::Web).unwrap();
    let glue = root.join("bridge.mjs");
    std::fs::write(&glue, &web.artifacts["bridge.js"].contents).unwrap();
    let driver = root.join("driver.mjs");
    std::fs::write(&driver, DRIVER).unwrap();
    let fixtures = conformance.join("tests/fixtures/one-time-execution");
    let output = run(
        Command::new("node")
            .arg(&driver)
            .arg(&glue)
            .arg(&wasm)
            .arg(&fixtures),
        "actual browser execution",
    );
    let results: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let answers = results["execution"].as_array().unwrap();
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(answers.len(), 20);
    assert_eq!(manifest.len(), answers.len());
    for (expected, actual) in manifest.iter().zip(answers) {
        let case = expected["case"].as_str().unwrap();
        assert_eq!(actual["case"], case);
        let answer = &actual["answer"];
        assert_eq!(
            answer["redacted"], true,
            "{case} observed plaintext in evidence"
        );
        assert_eq!(answer["total"], 1, "{case}");
        for field in ["status", "counts", "callback_trace"] {
            assert_eq!(answer[field], expected[field], "{case} {field}");
        }
        assert!(
            answer["diagnostics"]
                .as_str()
                .unwrap()
                .contains(expected["required_code"].as_str().unwrap()),
            "{case} fixed code"
        );
        let suite: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join(expected["suite"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let ids: Vec<_> = suite["scenarios"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            answer["scenario_ids"],
            serde_json::json!(ids),
            "{case} identities"
        );
    }
    additional_answers(&results, &fixtures);
}

fn additional_answers(results: &serde_json::Value, fixtures: &Path) {
    let root = fixtures.parent().unwrap();
    let window_suite = std::fs::read_to_string(root.join("one-time-windows/suite.json")).unwrap();
    compare_additional(
        &results["windows"],
        &root.join("one-time-windows/manifest.json"),
        &window_suite,
        2,
    );
    let resource_suite = std::fs::read_to_string(fixtures.join("view.json")).unwrap();
    compare_additional(
        &results["resources"],
        &root.join("one-time-resources.json"),
        &resource_suite,
        13,
    );
    let field_suite = std::fs::read_to_string(root.join("one-time-fields/suite.json")).unwrap();
    compare_additional(
        &results["fields"],
        &root.join("one-time-fields/manifest.json"),
        &field_suite,
        4,
    );
    let carrier: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("one-time-coverage/input.json")).unwrap())
            .unwrap();
    compare_additional(
        &results["coverage"],
        &root.join("one-time-coverage/manifest.json"),
        carrier["suite_json"].as_str().unwrap(),
        4,
    );
}

fn compare_additional(
    answers: &serde_json::Value,
    manifest_path: &Path,
    suite: &str,
    expected_len: usize,
) {
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(manifest_path).unwrap()).unwrap();
    let answers = answers.as_array().unwrap();
    assert_eq!(manifest.len(), expected_len);
    assert_eq!(answers.len(), manifest.len());
    let suite: serde_json::Value = serde_json::from_str(suite).unwrap();
    let ids: Vec<_> = suite["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    for (expected, actual) in manifest.iter().zip(answers) {
        let case = expected["case"].as_str().unwrap();
        assert_eq!(actual["case"], case);
        let answer = &actual["answer"];
        assert_eq!(answer["redacted"], true, "{case} resource redaction");
        assert_eq!(answer["total"], 1, "{case}");
        for field in ["status", "callback_trace"] {
            assert_eq!(answer[field], expected[field], "{case} {field}");
        }
        for count in ["passed", "failed", "skipped", "unsupported", "error"] {
            assert_eq!(
                answer["counts"][count], expected["counts"][count],
                "{case} {count}"
            );
        }
        let code = if let Some(code) = expected["required_code"].as_str() {
            code
        } else if expected["status"] == "passed" {
            "ESS-CF-DISCLOSURE"
        } else {
            "ESS-CF-TARGET"
        };
        assert!(
            answer["diagnostics"].as_str().unwrap().contains(code),
            "{case} fixed code"
        );
        assert_eq!(
            answer["scenario_ids"],
            serde_json::json!(ids),
            "{case} identities"
        );
    }
}
