//! The model-based explorer offers an external refusal answered before the row (`before_row: true`,
//! ess/25, beyond10x/ess#534) before existence and the held state, in both emitted languages.
//!
//! The fixture is `tests/fixtures/explore-before-row.yaml`: `SignIn` checks a credential before it
//! reads the seat (`bad-credential`), refuses a seat no row carries (`no-seat`) and selects by the
//! seat's held state. The targets are `explore-before-row-target.mjs` and
//! `explore_before_row_target.go`, run by the external-branch drivers: one asks the arranged
//! credential check first, the other reads the seat first and so answers `no-seat` for an identity
//! no seat carries even where the check was arranged.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

mod support_scratch;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/explore-before-row.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-before-row.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-before-row.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.select_fresh_format();
    suite
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn scratch(label: &str) -> support_scratch::Scratch {
    let root = support_scratch::Scratch::adopt(std::env::temp_dir().join(format!(
        "ess-explore-before-row-{}-{label}",
        std::process::id()
    )));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");
    root
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, contents).expect("writable");
}

fn printed(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// What one lane reported: each case's result and assertion, and the runner log.
#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Option<Value>>,
    asserts: BTreeMap<String, String>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json")))
            .ok()
            .map(|text| serde_json::from_str(&text).expect("the result is JSON"));
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        results.insert(name.clone(), json);
        asserts.insert(name, assert.trim_end().to_owned());
    }
    Lane {
        results,
        asserts,
        log,
    }
}

fn typescript(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    // The external-branch driver runs whichever double is copied in under its target's name.
    for (from, to) in [
        (
            "explore-before-row-target.mjs",
            "explore-external-target.mjs",
        ),
        ("explore-external-driver.mjs", "explore-external-driver.mjs"),
    ] {
        std::fs::copy(fixture(from), ts.join(to)).expect("the fixture copies");
    }
    write(
        &ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    );
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .expect("`tsc` is on PATH: the explorer lane does not skip");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "explore-external-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let line = format!("ok {} - {}", index + 1, case["name"].as_str().unwrap());
        assert!(
            log.lines().any(|it| it == line),
            "`{line}` is not in the node log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/explorebeforerow\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_before_row_target.go"),
        module.join("essconform/explore_external_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_external_driver_test.go"),
        module.join("essconform/explore_external_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExploreExternal",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for case in cases.as_array().unwrap() {
        let line = format!(
            "--- PASS: TestExploreExternal/{}",
            case["name"].as_str().unwrap()
        );
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

fn cases() -> Value {
    json!([
        {"name": "before-row"},
        {"name": "row-first", "mode": "row-first"},
    ])
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = scratch("lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn result<'a>(lane: &'a Lane, name: &str) -> &'a Value {
    lane.results[name].as_ref().unwrap_or_else(|| {
        panic!(
            "`{name}` returned no result: {}\n{}",
            lane.asserts[name], lane.log
        )
    })
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("a list, not {value}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

/// A double that asks the arranged check before it reads the seat agrees with the model, and the
/// explorer reaches the external branch.
#[test]
fn a_double_asking_the_check_before_the_row_agrees_and_reaches_it() {
    for lane in [&lanes().0, &lanes().1] {
        let found = result(lane, "before-row");
        assert_eq!(lane.asserts["before-row"], "ok", "{found}");
        assert!(found.get("failure").is_none(), "{found}");
        assert!(
            strings(&found["reached"]).contains(&"exploreext.desk.SignIn/bad-credential"),
            "{found}"
        );
        assert_eq!(
            found["external"],
            json!([{
                "outcome": "exploreext.desk.SignIn/bad-credential",
                "cause": "the secret is not the digest the service derives",
                "reach": "reached"
            }]),
            "{found}"
        );
    }
}

/// A double that reads the seat first answers `no-seat` for an identity no seat carries, where the
/// model answers the arranged credential check: the explorer reports the disagreement.
#[test]
fn a_double_reading_the_row_first_is_a_disagreement() {
    for lane in [&lanes().0, &lanes().1] {
        let found = result(lane, "row-first");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "outcome: the target answered `no-seat`, the specification says `bad-credential`"
            ),
            "{found}"
        );
        assert!(
            lane.asserts["row-first"].starts_with("failed: explore: seed "),
            "{}",
            lane.asserts["row-first"]
        );
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_case() {
    let (typescript, go) = lanes();
    for (name, result) in &typescript.results {
        assert_eq!(
            result, &go.results[name],
            "`{name}` differs between TypeScript and Go\nTypeScript: {:?}\nGo: {:?}",
            result, go.results[name]
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
    }
}

// ---- an external refusal answered before a related row (beyond10x/ess#554) -------------------

/// `invalid-zone` refuses a zone the provider cannot parse before the job's name is looked up
/// (`before_row: true`); `duplicate-name` refuses a name a stored job carries.
const JOBS: &str = "format: ess/37
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobName, kind: newtype, of: String}
entities:
  - name: demo.jobs.Job
    identity: {name: name, type: demo.jobs.JobName}
    fields:
      - {name: zone, type: String}
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
errors:
  - {name: demo.jobs.InvalidZone}
  - {name: demo.jobs.Duplicate}
events:
  - name: demo.jobs.JobStored
    fields:
      - {name: name, type: demo.jobs.JobName}
commands:
  - name: demo.jobs.StoreJob
    input:
      - {name: name, type: demo.jobs.JobName}
      - {name: zone, type: String}
    outcomes:
      - name: invalid-zone
        external: the zone id does not parse
        before_row: true
        error: demo.jobs.InvalidZone
      - name: duplicate-name
        when_related: {entity: demo.jobs.Job, where: name == input.name, exists: true}
        error: demo.jobs.Duplicate
      - name: stored
        creates: demo.jobs.Job
        instance: name
        sets: {zone: input.zone}
        emits: [demo.jobs.JobStored]
        payload:
          demo.jobs.JobStored: {name: input.name}
";

const INVALID_ZONE: &str = "demo.jobs.StoreJob/outcome/invalid-zone";

fn jobs_ir(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("jobs.yaml", text);
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let specification = Specification::assemble(vec![(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

/// The `StoreJob` invocations of the `invalid-zone` scenario that follow a forced provider, each
/// with whether a stored job carried the name it sends.
fn forced_sends(suite: &ConformanceSuite) -> Vec<bool> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == INVALID_ZONE)
        .unwrap_or_else(|| panic!("no scenario {INVALID_ZONE}"))
        .1;
    let mut stored: Vec<ess_conformance::ScenarioValue> = Vec::new();
    let mut forced = false;
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        match step {
            ess_conformance::ScenarioStep::ConfigureExternalOutcome { .. } => forced = true,
            ess_conformance::ScenarioStep::ExecuteCommand { input, .. } => {
                let name = input.get("name").cloned().expect("a name");
                let outcome = match steps.peek() {
                    Some(ess_conformance::ScenarioStep::ExpectOutcome { outcome }) => {
                        outcome.outcome.to_string()
                    }
                    _ => String::new(),
                };
                if forced {
                    assert_eq!(outcome, "invalid-zone", "{scenario:#?}");
                    out.push(stored.contains(&name));
                    forced = false;
                } else if outcome == "stored" {
                    stored.push(name);
                }
            }
            _ => {}
        }
    }
    out
}

fn failing_on_interpreted(suite: &ConformanceSuite, model: &str) -> Vec<String> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(suite).unwrap();
    ess_conformance::Runner::for_suite(suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(jobs_ir(model)),
        )
        .into_report()
        .scenarios
        .into_iter()
        .filter(|scenario| scenario.status != ess_conformance::report::Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

#[test]
fn an_external_refusal_before_the_row_is_sent_beside_a_present_related_row() {
    let synthesis = ess_conformance::synthesize(&jobs_ir(JOBS));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    assert!(forced_sends(&suite).contains(&true), "{suite:#?}");
    assert_eq!(failing_on_interpreted(&suite, JOBS), Vec::<String>::new());
    // A service that looks the name up first: `duplicate-name` declared before the external
    // refusal, which then answers at its own step.
    let row_first = JOBS.replace("        before_row: true\n", "");
    let refusal = "      - name: duplicate-name\n        when_related: {entity: demo.jobs.Job, where: name == input.name, exists: true}\n        error: demo.jobs.Duplicate\n";
    let row_first = row_first.replace(refusal, "").replace(
        "      - name: invalid-zone\n",
        &format!("{refusal}      - name: invalid-zone\n"),
    );
    assert!(row_first.find("duplicate-name") < row_first.find("invalid-zone"));
    assert_eq!(
        failing_on_interpreted(&suite, &row_first),
        vec![INVALID_ZONE.to_owned()]
    );
}

#[test]
fn no_overlap_scenario_without_a_related_row_refusal() {
    let refusal = "      - name: duplicate-name\n        when_related: {entity: demo.jobs.Job, where: name == input.name, exists: true}\n        error: demo.jobs.Duplicate\n";
    let alone = JOBS.replace(refusal, "");
    assert_ne!(alone, JOBS);
    let synthesis = ess_conformance::synthesize(&jobs_ir(&alone));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert!(!forced_sends(&synthesis.suite).contains(&true));
}
