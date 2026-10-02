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

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}

fn run(request: &str) -> String {
    let request: Value = serde_json::from_str(request).unwrap();
    let mode: Mode = serde_json::from_value(request["mode"].clone()).unwrap();
    let suite = AdmittedSuite::from_json(request["suite"].as_str().unwrap()).unwrap();
    let target = Service::new(mode);
    let report = Runner::for_suite(suite.suite()).run_admitted(&suite, &target);
    let counts = ess_conformance::counts::CountReport::from_run(&report, &suite).unwrap();
    let diagnostics = serde_json::to_string(&report.scenarios).unwrap();
    let count_bytes = counts.to_canonical_json().unwrap();
    let mut all_counts = serde_json::to_value(counts.counts()).unwrap();
    let total = all_counts.as_object_mut().unwrap().remove("total").unwrap();
    let answer = json!({
        "status": report.scenarios[0].status,
        "scenario_ids": report.scenarios.iter().map(|s| &s.scenario).collect::<Vec<_>>(),
        "counts": all_counts, "total": total, "diagnostics": diagnostics,
        "callback_trace": target.trace(), "count_report": count_bytes, "redacted": true,
    }).to_string();
    // Captures stay inside WASM. Even the test result carries no captured plaintext.
    for value in target.returned_plaintexts().into_iter().chain([FIRST.to_owned()]) {
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
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
const [glue, wasm, fixtures] = process.argv.slice(2);
const {open} = await import(pathToFileURL(glue));
const system = await open(readFileSync(wasm));
const manifest = JSON.parse(readFileSync(join(fixtures, 'manifest.json'), 'utf8'));
const answers = manifest.map(item => ({
  case: item.case,
  answer: system.request({mode: item.case, suite: readFileSync(join(fixtures, item.suite), 'utf8')}),
}));
process.stdout.write(JSON.stringify(answers));
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
    std::fs::write(
        root.join("src/lib.rs"),
        format!("#[allow(dead_code)]\n#[path = {support_path}]\nmod support;\n{HOST}"),
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
    let answers: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(answers.len(), 19);
    assert_eq!(manifest.len(), answers.len());
    for (expected, actual) in manifest.iter().zip(&answers) {
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
}
