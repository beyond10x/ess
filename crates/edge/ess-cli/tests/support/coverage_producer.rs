use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    coverage::{AdmittedInput, Origins, Scope, SuiteReference},
    coverage_build::{self, CoverageSource},
    AdmittedSuite, ConformanceSuite, CountReport, ScenarioId,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source as SpecSource,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};
#[path = "coverage_producer_go.rs"]
mod go;
#[path = "coverage_producer_target.rs"]
mod target;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coverage-producers")
}
fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_string_pretty(value).unwrap() + "\n").unwrap();
}
fn text(value: &Value) -> &str {
    value.as_str().unwrap()
}
fn list(value: &Value) -> &[Value] {
    value.as_array().unwrap()
}
fn files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            result.extend(files(&entry.path()));
        } else if kind.is_file() {
            result.push(entry.path());
        } else {
            panic!(
                "fixture/source manifest must not dereference {}",
                entry.path().display()
            );
        }
    }
    result.sort();
    result
}
fn hash(path: &Path) -> String {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .into()
}
fn command(directory: &Path, label: &str, command: &mut Command) -> Output {
    fs::create_dir_all(directory).unwrap();
    fs::write(
        directory.join(format!("{label}.command")),
        format!("{command:?}\n"),
    )
    .unwrap();
    let start = Instant::now();
    let output = command.output().unwrap();
    fs::write(directory.join(format!("{label}.stdout")), &output.stdout).unwrap();
    fs::write(directory.join(format!("{label}.stderr")), &output.stderr).unwrap();
    write_json(
        &directory.join(format!("{label}.receipt.json")),
        &json!({
            "command":format!("{command:?}"), "exit":output.status.code(),
            "success":output.status.success(), "elapsed_ns":start.elapsed().as_nanos().to_string(),
        }),
    );
    output
}
fn setup(profile: &str) -> (PathBuf, Value, BTreeMap<String, AdmittedInput>) {
    let base = std::env::var_os("ESS_COVERAGE_EXPORT_ROOT").map_or_else(
        || std::env::temp_dir().join(format!("ess-coverage-producers-{}", std::process::id())),
        PathBuf::from,
    );
    let directory = base.join(profile);
    fs::create_dir_all(&directory).unwrap();
    let fixture = fixtures();
    let plan = read_json(&fixture.join("semantic-plan.json"));
    assert_eq!(
        hash(&fixture.join("semantic-plan.json")),
        "5a378b14f7747ce7b3f1eac9b6d6e8c5962c02f81e4e116f7e74d478bcae6ef1"
    );
    for entry in list(&read_json(&fixture.join("input-catalog.json"))["files"]) {
        assert_eq!(
            hash(&fixture.join(text(&entry["path"]))),
            text(&entry["sha256"])
        );
    }
    fs::copy(
        fixture.join("semantic-plan.json"),
        directory.join("semantic-plan.json"),
    )
    .unwrap();
    source_receipt(&directory);
    let structures = structures(&plan, &directory);
    (directory, plan, structures)
}
fn source_receipt(directory: &Path) {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut manifest = BTreeMap::new();
    let mut sources = files(&repository.join("crates"));
    sources.extend([repository.join("Cargo.toml"), repository.join("Cargo.lock")]);
    for name in [
        ".cargo",
        "rust-toolchain.toml",
        "rust-toolchain",
        "build.rs",
    ] {
        let path = repository.join(name);
        if path.is_dir() {
            sources.extend(files(&path));
        } else if path.is_file() {
            sources.push(path);
        }
    }
    sources.sort();
    for path in sources {
        let relative = path.strip_prefix(&repository).unwrap();
        let snapshot = directory.join("source-snapshot").join(relative);
        fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        fs::copy(&path, &snapshot).unwrap();
        let digest = hash(&path);
        assert_eq!(hash(&snapshot), digest);
        manifest.insert(relative.to_str().unwrap().to_owned(), digest);
    }
    write_json(&directory.join("source-manifest.json"), &json!(manifest));
    let executable = std::env::current_exe().unwrap();
    write_json(
        &directory.join("producer.json"),
        &json!({
            "runtime":concat!("ess-conformance/",env!("CARGO_PKG_VERSION")),
            "test_executable":executable, "test_executable_sha256":hash(&executable),
            "test_argv":std::env::args().collect::<Vec<_>>(), "cli_executable":env!("CARGO_BIN_EXE_ess"),
            "cli_sha256":hash(Path::new(env!("CARGO_BIN_EXE_ess"))),
            "plan_sha256":hash(&fixtures().join("semantic-plan.json")), "source_manifest":"source-manifest.json"
        }),
    );
    for (label, program, args) in [
        ("rustc", "rustc", vec!["-Vv"]),
        ("go-version", "go", vec!["version"]),
    ] {
        assert!(command(directory, label, Command::new(program).args(args))
            .status
            .success());
    }
}
fn model(path: &Path) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for file in files(path) {
        let label = file.strip_prefix(path).unwrap().to_str().unwrap();
        let original = fs::read_to_string(&file).unwrap();
        let raw = RawSpecFile::parse(&original).unwrap();
        sources.insert(label, original);
        parsed.push((SpecSource::new(label), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}
fn authored(path: &Value) -> Vec<CoverageSource> {
    if path.is_null() {
        return Vec::new();
    }
    let root = fixtures().join(text(path));
    files(&root)
        .iter()
        .map(|file| {
            CoverageSource::new(
                file.strip_prefix(&root).unwrap().to_str().unwrap(),
                fs::read_to_string(file).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
fn structures(plan: &Value, directory: &Path) -> BTreeMap<String, AdmittedInput> {
    let mut result: BTreeMap<String, AdmittedInput> = BTreeMap::new();
    for case in list(&plan["structural_cases"]) {
        let id = text(&case["id"]);
        let input = if let Some(parent) = list(&case["parent_cases_nearest_first"]).first() {
            let ids: Vec<ScenarioId> =
                serde_json::from_value(case["selected_ids"].clone()).unwrap();
            result[text(parent)].select(&ids).unwrap()
        } else {
            fresh_structure(case)
        };
        check_structure(case, &input, &result);
        let out = directory.join("structures").join(id);
        fs::create_dir_all(&out).unwrap();
        transport(&out, &input);
        result.insert(id.into(), input);
    }
    assert_eq!(result.len(), 17);
    result
}
fn fresh_structure(case: &Value) -> AdmittedInput {
    let ir = model(&fixtures().join(text(&case["model_input"])));
    let selection = &case["inventory_expectation"]["selection"];
    let scope: Scope = serde_json::from_value(selection["scope"].clone()).unwrap();
    let origins: Origins = serde_json::from_value(selection["origins"].clone()).unwrap();
    let sources = authored(&case["authored_input"]);
    match text(&case["id"]) {
        "S16-final-merge" => {
            let first = coverage_build::compile_sources(
                &ir,
                &authored(&json!("inputs/authored/merge-first")),
            )
            .unwrap();
            let second = coverage_build::compile_sources(
                &ir,
                &authored(&json!("inputs/authored/merge-second")),
            )
            .unwrap();
            assert_eq!((first.accepted(), second.accepted()), (1, 1));
            assert_eq!(
                (first.refusals().count(), second.refusals().count()),
                (0, 0)
            );
            coverage_build::merge_batches(&ir, &[first, second], scope, origins).unwrap()
        }
        "S17-known-origin-selection" => {
            coverage_build::build_with_known_generated(&ir, &sources, scope, origins).unwrap()
        }
        "S08-unknown" => {
            let built = coverage_build::build(&ir, &sources, scope, origins).unwrap();
            let mut document: Value =
                serde_json::from_str(built.selected().original_json()).unwrap();
            document["coverage"]["knowledge"] = json!("unknown");
            AdmittedInput::from_suite(AdmittedSuite::from_json(&document.to_string()).unwrap())
                .unwrap()
        }
        _ => coverage_build::build(&ir, &sources, scope, origins).unwrap(),
    }
}
fn expected_inventory(case: &Value, structures: &BTreeMap<String, AdmittedInput>) -> Value {
    let resolutions_path = fixtures().join("root-resolutions.json");
    assert_eq!(
        hash(&resolutions_path),
        "6feb6754888aefd7fbd9b371fdc09ed42ec30bbcbef3962176d7e1a9f8ca9f8b"
    );
    let resolutions = read_json(&resolutions_path);
    let mut expected = case["inventory_expectation"].clone();
    if let Some(parent) =
        expected["selection"]["filter"]["parent"]["deferred_exact_reference_to_case"].as_str()
    {
        let reference =
            serde_json::to_value(SuiteReference::of(structures[parent].selected())).unwrap();
        expected["selection"]["filter"]["parent"] = reference;
    }
    for refusal in expected["refused"].as_array_mut().unwrap() {
        let key = format!("{}|{}", text(&refusal["code"]), text(&refusal["scenario"]));
        let message = text(&resolutions["D1"]["messages"][&key]);
        let indented = text(&refusal["message"]["required_original_cause"])
            .lines()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(message.contains(&indented));
        refusal["message"] = json!(message);
        if refusal["subject"].get("independently_resolve").is_some() {
            refusal["subject"] = Value::Null;
        }
    }
    expected
}
fn check_structure(
    case: &Value,
    input: &AdmittedInput,
    structures: &BTreeMap<String, AdmittedInput>,
) {
    let id = text(&case["id"]);
    assert_eq!(
        serde_json::to_value(input.selected().coverage().unwrap()).unwrap(),
        expected_inventory(case, structures),
        "{id} inventory"
    );
    let actual_ids: Vec<_> = input
        .selected()
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(json!(actual_ids), case["selected_ids"], "{id} IDs");
    for key in [
        "system",
        "specification_version",
        "spec_digest",
        "contract_digest",
    ] {
        let actual = serde_json::to_value(&input.selected().suite().provenance).unwrap();
        assert_eq!(actual[key], case["model_provenance"][key], "{id}/{key}");
    }
    let oracle = ConformanceSuite::from_json(
        &fs::read_to_string(fixtures().join(text(&case["scenario_definition_oracle"]))).unwrap(),
    )
    .unwrap();
    for (scenario_id, scenario) in &input.selected().suite().scenarios {
        assert_eq!(
            scenario, &oracle.scenarios[scenario_id],
            "{id}/{scenario_id} full definition"
        );
    }
    assert_eq!(
        input.parents().len(),
        list(&case["parent_cases_nearest_first"]).len()
    );
    for (parent, name) in input
        .parents()
        .iter()
        .zip(list(&case["parent_cases_nearest_first"]))
    {
        assert_eq!(
            parent.original_json(),
            structures[text(name)].selected().original_json()
        );
    }
}
fn transport(directory: &Path, input: &AdmittedInput) {
    fs::write(
        directory.join("suite.json"),
        input.selected().original_json(),
    )
    .unwrap();
    fs::write(
        directory.join("input.json"),
        input.document().to_canonical_json().unwrap(),
    )
    .unwrap();
    let mut parents = Vec::new();
    for (index, parent) in input.parents().iter().enumerate() {
        let name = format!("parent-{index}.json");
        fs::write(directory.join(&name), parent.original_json()).unwrap();
        parents.push(json!({"path":name,"sha256":hash(&directory.join(&name)),"reference":SuiteReference::of(parent)}));
    }
    write_json(
        &directory.join("transport.json"),
        &json!({
            "suite":{"path":"suite.json","sha256":hash(&directory.join("suite.json")),"reference":SuiteReference::of(input.selected())},
            "input":{"path":"input.json","sha256":hash(&directory.join("input.json"))},"parents":parents
        }),
    );
}
fn check_report(
    instance: &Value,
    original: &str,
    input: &AdmittedInput,
    completed_at: u64,
) -> CountReport {
    let report = CountReport::from_json(original, input.selected()).unwrap();
    let value: Value = serde_json::from_str(original).unwrap();
    for (actual, expected) in [
        ("counts", "expected_counts"),
        ("outcomes", "expected_outcomes"),
        ("execution_status", "expected_execution_status"),
        ("conformance_status", "expected_conformance_status"),
        ("implementation", "expected_implementation"),
        ("specification", "expected_specification"),
        ("spec_digest", "expected_model_digest"),
    ] {
        assert_eq!(
            value[actual], instance[expected],
            "{} / {actual}",
            instance["id"]
        );
    }
    assert_eq!(value["producer_profile"], instance["producer_profile"]);
    if let Some(refused) = value["coverage"]["refused"]
        .as_array()
        .filter(|r| r.len() > 1)
    {
        let mut coalesced = refused.clone();
        coalesced.dedup();
        if coalesced.len() < refused.len() {
            let mut changed = value.clone();
            changed["coverage"]["counts"]["refused"] = json!(coalesced.len());
            changed["coverage"]["refused"] = json!(coalesced);
            assert!(CountReport::from_json(&changed.to_string(), input.selected()).is_err());
            let mut changed = value.clone();
            changed["coverage"]["refused"][1]["message"] =
                json!(format!("{} (2)", text(&refused[1]["message"])));
            assert!(CountReport::from_json(&changed.to_string(), input.selected()).is_err());
        }
    }
    assert_eq!(report.completed_at(), completed_at, "independent clock");
    report
}
pub fn export_rust() {
    let (directory, plan, structures) = setup("rust");
    let mut executed = 0;
    for instance in list(&plan["requested_report_instances"])
        .iter()
        .filter(|i| i["producer_profile"] == "rust-scenario-status/1")
    {
        let input = &structures[text(&instance["structure"])];
        let out = directory.join(text(&instance["id"]));
        fs::create_dir_all(&out).unwrap();
        transport(&out, input);
        target::run(instance, input, &out);
        executed += 1;
    }
    assert_eq!(executed, 24);
    println!("actual Rust producer exports: {executed}");
}
pub fn export_go() {
    let (directory, plan, structures) = setup("go");
    go::export(&directory, &plan, &structures);
}

pub fn export_controls() {
    let (directory, plan, structures) = setup("controls");
    for control in list(&plan["requested_refusal_controls"]) {
        let input = &structures[text(&control["structure"])];
        let out = directory.join(text(&control["id"]));
        fs::create_dir_all(&out).unwrap();
        if control["profile"] == "go" {
            go::refuse(&out, input, control);
        } else {
            transport(&out, input);
            let output = command(
                &out,
                "cli-run",
                Command::new(env!("CARGO_BIN_EXE_ess"))
                    .args(["conform", "run", "--suite-input"])
                    .arg(out.join("input.json"))
                    .args([
                        "--target",
                        "billing",
                        "--report-format",
                        "1",
                        "--allow-incomplete",
                        "--format",
                        "json",
                    ]),
            );
            assert!(!output.status.success(), "{output:?}");
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr)
                .contains("suite/5 requires explicit --report-format 2 before execution"));
            assert!(!out.join("report.json").exists());
            write_json(
                &out.join("fixture.json"),
                &json!({"independent_expected":control,
                "report_expected":false,"command_receipt":"cli-run.receipt.json",
                "callback_observation":"CLI pair gate refuses before its target-construction closure; coverage::tests::suite5_pair_refusal_precedes_target_construction measures the same closure boundary",
                "producer_receipt":"../producer.json","transport":"transport.json"}),
            );
        }
    }
}
