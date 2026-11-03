//! The explorer and the interpreted target expect the same branch when an `external:` cause is
//! arranged on a command whose held state another branch answers
//! (`story:explorer-and-interpreter-agree-on-external-cause-for-stored-record`).
//!
//! The fixture is `tests/fixtures/explore-held-state-external.yaml`: `Nudge` refuses a parked
//! ticket (`parked`, `when_subject_state:`), is refused when the gateway bounces (`bounced`, a plain
//! `external:`) and otherwise nudges. The interpreter reads the held state (step 3 of the
//! precedence order) before a plain external branch (step 5), so an arranged bounce never answers a
//! parked ticket. The explorer offered `bounced` beside `parked` and, once it was arranged, expected
//! it there.
//!
//! The targets are `explore-held-state-external-target.mjs` and
//! `explore_held_state_external_target.go`, run by the external-branch drivers. Each logs every
//! `Nudge` it answered; every logged step is replayed through the interpreter, so a passing
//! exploration is one whose every arranged cause the interpreter answers as the explorer expected.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

mod support_scratch;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/explore-held-state-external.yaml");
const NUDGE: &str = "explorehx.desk.Nudge";
const NO_TICKET: &str = "00000000-0000-4000-8000-0000000000ff";

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-held-state-external.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-held-state-external.yaml"), raw)])
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

// ---- the interpreter ----------------------------------------------------------------------------

fn interpreted(
    ir: &EssIr,
    store: &Store,
    command: &str,
    input: &[(&str, Node)],
    arranged: &str,
) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let externals = if arranged.is_empty() {
        Externals::Withheld
    } else {
        Externals::Forced(OutcomeName::new(arranged).expect("an outcome name"))
    };
    let mut steps = execute(ir, store, &command.parse().unwrap(), &input, &externals)
        .unwrap_or_else(|error| panic!("{command} {input:?} is interpreted: {error}"));
    assert_eq!(steps.len(), 1, "{command} has one selected outcome");
    steps.remove(0)
}

fn answered(step: &Step) -> String {
    let outcome = step
        .outcome
        .as_ref()
        .expect("a declared outcome")
        .to_string();
    outcome
        .rsplit_once('/')
        .map_or(outcome.clone(), |(_, name)| name.to_owned())
}

/// A store holding one ticket resting in `state`, and the ticket's identity.
fn ticket_in(ir: &EssIr, state: &str) -> (Store, String) {
    let opened = interpreted(
        ir,
        &Store::default(),
        "explorehx.desk.OpenTicket",
        &[("label", Node::Text("a".into()))],
        "",
    );
    assert_eq!(answered(&opened), "opened");
    let id = opened
        .events
        .iter()
        .find_map(|event| event.payload.values().find_map(Node::as_text))
        .expect("the opened event carries the identity")
        .to_owned();
    match state {
        "Open" => (opened.next, id),
        "Parked" => {
            let parked = interpreted(
                ir,
                &opened.next,
                "explorehx.desk.Park",
                &[("ticket_id", Node::Text(id.clone()))],
                "",
            );
            assert_eq!(answered(&parked), "parked-now");
            (parked.next, id)
        }
        other => panic!("no ticket rests in `{other}`"),
    }
}

/// The branch the interpreter answers a `Nudge` of a ticket resting in `state` (empty: an identity
/// no ticket carries) with `arranged` forced (empty: none).
fn interpreter_nudge(ir: &EssIr, state: &str, arranged: &str) -> String {
    let (store, id) = if state.is_empty() {
        (Store::default(), NO_TICKET.to_owned())
    } else {
        ticket_in(ir, state)
    };
    answered(&interpreted(
        ir,
        &store,
        NUDGE,
        &[("ticket_id", Node::Text(id))],
        arranged,
    ))
}

#[test]
fn the_interpreter_answers_the_held_state_before_an_arranged_external_cause() {
    let ir = ir();
    assert_eq!(interpreter_nudge(&ir, "Parked", "bounced"), "parked");
    assert_eq!(interpreter_nudge(&ir, "Parked", ""), "parked");
    assert_eq!(interpreter_nudge(&ir, "Open", "bounced"), "bounced");
    assert_eq!(interpreter_nudge(&ir, "Open", ""), "nudged");
    assert_eq!(interpreter_nudge(&ir, "", "bounced"), "no-ticket");
}

// ---- the explorer lanes -------------------------------------------------------------------------

#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Option<Value>>,
    asserts: BTreeMap<String, String>,
    /// Every `Nudge` the target answered, by mode: (state, arranged, answered).
    answers: BTreeMap<String, Vec<(String, String, String)>>,
    log: String,
}

fn read_lane(out: &Path, answers: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    let mut logged = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json")))
            .ok()
            .map(|text| serde_json::from_str(&text).expect("the result is JSON"));
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        results.insert(name.clone(), json);
        asserts.insert(name, assert.trim_end().to_owned());
        let mode = case["mode"].as_str().unwrap_or("held-first").to_owned();
        let text =
            std::fs::read_to_string(answers.join(format!("{mode}.jsonl"))).unwrap_or_default();
        let steps = text
            .lines()
            .map(|line| {
                let step: Value = serde_json::from_str(line).expect("an answer line is JSON");
                let field = |name: &str| step[name].as_str().expect("a text field").to_owned();
                (field("state"), field("arranged"), field("answered"))
            })
            .collect();
        logged.insert(mode, steps);
    }
    Lane {
        results,
        asserts,
        answers: logged,
        log,
    }
}

fn typescript(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    for (from, to) in [
        (
            "explore-held-state-external-target.mjs",
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
    let answers = root.join("answers-typescript");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::create_dir_all(&answers).unwrap();
    let run = Command::new("node")
        .args(["--test", "explore-external-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env("ESS_EXPLORE_ANSWERS", &answers)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, &answers, cases, log)
}

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploreheldexternal\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_held_state_external_target.go"),
        module.join("essconform/explore_external_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_external_driver_test.go"),
        module.join("essconform/explore_external_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    let answers = root.join("answers-go");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::create_dir_all(&answers).unwrap();
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
        .env("ESS_EXPLORE_ANSWERS", &answers)
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, &answers, cases, log)
}

fn cases() -> Value {
    json!([
        {"name": "held-first"},
        {"name": "external-first", "mode": "external-first"},
    ])
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = support_scratch::Scratch::new(format!(
            "ess-explore-held-state-external-{}",
            std::process::id()
        ));
        (typescript(&root, &ir, &cases), go(&root, &ir, &cases))
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

/// A target that answers in the interpreter's order passes, every `Nudge` it answered is the
/// interpreter's answer for the same state and the same arranged cause, and among them is an
/// arranged bounce on a parked ticket, answered `parked`.
#[test]
fn a_target_in_the_interpreters_order_agrees_with_the_explorer_for_every_arranged_cause() {
    let ir = ir();
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "held-first");
        assert_eq!(lane.asserts["held-first"], "ok", "{language}: {found}");
        let reached = strings(&found["reached"]);
        for outcome in ["parked", "bounced", "nudged"] {
            assert!(
                reached.contains(&format!("{NUDGE}/{outcome}").as_str()),
                "{language}: `{outcome}` is reached\n{found}"
            );
        }
        let steps = &lane.answers["held-first"];
        let distinct: BTreeSet<&(String, String, String)> = steps.iter().collect();
        for (state, arranged, answer) in &distinct {
            assert_eq!(
                &interpreter_nudge(&ir, state, arranged),
                answer,
                "{language}: the interpreter answers a `Nudge` of a ticket in `{state}` with \
                 `{arranged}` arranged as the explorer expected"
            );
        }
        assert!(
            distinct.contains(&(
                "Parked".to_owned(),
                "bounced".to_owned(),
                "parked".to_owned()
            )),
            "{language}: an arranged bounce on a parked ticket is explored: {distinct:?}"
        );
    }
}

/// A target that asks the arranged gateway before it reads the held state answers `bounced` on a
/// parked ticket, where the interpreter answers `parked`: the explorer reports the disagreement.
#[test]
fn a_target_asking_the_gateway_before_the_held_state_is_a_disagreement() {
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "external-first");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "outcome: the target answered `bounced`, the specification says `parked`"
            ),
            "{language}: {found}"
        );
        assert!(
            lane.asserts["external-first"].starts_with("failed: explore: seed "),
            "{language}: {}",
            lane.asserts["external-first"]
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
