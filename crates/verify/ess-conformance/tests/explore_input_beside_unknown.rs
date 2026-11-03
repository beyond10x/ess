//! The interpreter and the explorer expect the same branch where an input refusal declared above an
//! unknown-instance branch and a related row's `exists: false` both hold
//! (`story:unknown-instance-and-input-guard-agree-on-order`), in both emitted languages.
//!
//! The fixture is `tests/fixtures/explore-input-beside-unknown.yaml` at `ess/29`. `Nudge` refuses
//! an amount over 100 (`too-much`, step 1 of the precedence order) and an identity no ticket
//! carries (`no-ticket`, step 2). `Route` refuses the same two and a desk no row carries
//! (`no-desk`, step 4). Each declares them in step order, the only order `ess/29` admits where
//! the guards can hold together (`ess-domain`'s `held_state_order.rs` pins the refusal of the
//! others). The interpreter answers both commands by the rule. The explorer leaves `Route` out,
//! as it leaves out every command with a `when_related:` branch. Exploring `Nudge` against a target
//! that asks in declaration order, it finds no draw it disagrees with and reaches every branch, and
//! a target asking for the ticket before the amount is caught.
//!
//! The targets are `explore-input-beside-unknown-target.mjs` and
//! `explore_input_beside_unknown_target.go`, run by the external-branch drivers.
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
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/explore-input-beside-unknown.yaml");
const NUDGE: &str = "exploreiu.desk.Nudge";
const ROUTE: &str = "exploreiu.desk.Route";
const NOBODY: &str = "00000000-0000-4000-8000-0000000000ff";

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-input-beside-unknown.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(
        Source::new("explore-input-beside-unknown.yaml"),
        raw,
    )])
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

fn interpreted(ir: &EssIr, store: &Store, command: &str, input: &[(&str, Node)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let mut steps = execute(
        ir,
        store,
        &command.parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
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

/// `store` after `command` creates a row, and the row's identity.
fn created(ir: &EssIr, store: &Store, command: &str, outcome: &str) -> (Store, String) {
    let step = interpreted(ir, store, command, &[("label", Node::Text("a".into()))]);
    assert_eq!(answered(&step), outcome);
    let id = step
        .events
        .iter()
        .find_map(|event| event.payload.values().find_map(Node::as_text))
        .expect("the event carries the identity")
        .to_owned();
    (step.next, id)
}

/// The branch the interpreter answers `Nudge` (`desk` `None`) or `Route` (`desk` set) of `amount`
/// with a ticket, and for `Route` a desk, that a row carries where its flag is set and an identity
/// no row carries otherwise.
fn interpreter_answer(ir: &EssIr, ticket: bool, desk: Option<bool>, amount: i64) -> String {
    let mut store = Store::default();
    let mut ticket_id = NOBODY.to_owned();
    let mut desk_id = NOBODY.to_owned();
    if ticket {
        (store, ticket_id) = created(ir, &store, "exploreiu.desk.OpenTicket", "opened");
    }
    if desk == Some(true) {
        (store, desk_id) = created(ir, &store, "exploreiu.desk.MakeDesk", "made");
    }
    let mut input = vec![
        ("ticket_id", Node::Text(ticket_id)),
        ("amount", Node::Number(amount.into())),
    ];
    let command = if desk.is_some() {
        input.push(("desk_id", Node::Text(desk_id)));
        ROUTE
    } else {
        NUDGE
    };
    answered(&interpreted(ir, &store, command, &input))
}

#[test]
fn the_interpreter_answers_the_input_refusal_before_an_unknown_ticket_and_a_missing_desk() {
    let ir = ir();
    for (ticket, desk, amount, expected) in [
        (false, None, 200, "too-much"),
        (false, None, 5, "no-ticket"),
        (true, None, 5, "nudged"),
        (false, Some(false), 200, "too-much"),
        (true, Some(false), 200, "too-much"),
        (false, Some(true), 200, "too-much"),
        (false, Some(false), 5, "no-ticket"),
        (true, Some(false), 5, "no-desk"),
        (true, Some(true), 5, "routed"),
    ] {
        assert_eq!(
            interpreter_answer(&ir, ticket, desk, amount),
            expected,
            "ticket held: {ticket}, desk held: {desk:?}, amount {amount}"
        );
    }
}

// ---- the explorer -------------------------------------------------------------------------------

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
    for (from, to) in [
        (
            "explore-input-beside-unknown-target.mjs",
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
    read_lane(&out, cases, log)
}

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploreinputbesideunknown\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_input_beside_unknown_target.go"),
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
    read_lane(&out, cases, log)
}

/// `(typescript, go)`.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        // `Route` is left out of exploration, so every case accepts that exclusion.
        let cases = json!([
            {"name": "declared", "allowExcluded": true},
            {"name": "ticket-first", "mode": "ticket-first", "allowExcluded": true},
        ]);
        let root = support_scratch::Scratch::new(format!(
            "ess-explore-input-beside-unknown-{}",
            std::process::id()
        ));
        (typescript(&root, &ir, &cases), go(&root, &ir, &cases))
    })
}

fn each_lane() -> [(&'static str, &'static Lane); 2] {
    let (typescript, go) = lanes();
    [("typescript", typescript), ("go", go)]
}

fn result<'a>(lane: &'a Lane, name: &str) -> &'a Value {
    lane.results[name].as_ref().unwrap_or_else(|| {
        panic!(
            "`{name}` returned no result: {}\n{}",
            lane.asserts[name], lane.log
        )
    })
}

#[test]
fn a_target_asking_in_declaration_order_agrees_with_the_explorer_and_reaches_every_branch() {
    for (language, lane) in each_lane() {
        let found = result(lane, "declared");
        assert_eq!(lane.asserts["declared"], "ok", "{language}: {found}");
        let reached: Vec<&str> = found["reached"]
            .as_array()
            .unwrap_or_else(|| panic!("a reached list, not {found}"))
            .iter()
            .map(|outcome| outcome.as_str().expect("a string"))
            .collect();
        for branch in ["too-much", "no-ticket", "nudged"] {
            assert!(
                reached.contains(&format!("{NUDGE}/{branch}").as_str()),
                "{language}: `{branch}` is reached: {found}"
            );
        }
        // The explorer's own limit, not this story's: a `when_related:` branch is not drawn.
        let excluded: Vec<(&str, &str)> = found["excluded"]
            .as_array()
            .unwrap_or_else(|| panic!("an exclusion list, not {found}"))
            .iter()
            .map(|exclusion| {
                (
                    exclusion["subject"].as_str().expect("a subject"),
                    exclusion["reason"].as_str().expect("a reason"),
                )
            })
            .collect();
        assert_eq!(
            excluded,
            [(ROUTE, "outcome `no-desk` has a `related` condition")],
            "{language}: {found}"
        );
    }
}

/// A target that asks for the ticket before the amount answers `no-ticket` where the specification
/// refuses the amount, and the explorer names both branches.
#[test]
fn a_target_asking_for_the_ticket_before_the_input_is_a_disagreement() {
    for (language, lane) in each_lane() {
        let assert = &lane.asserts["ticket-first"];
        assert!(
            assert.starts_with("failed: explore: seed "),
            "{language}: {assert}\n{}",
            lane.log
        );
        let failure = result(lane, "ticket-first")["failure"]["message"]
            .as_str()
            .unwrap_or_else(|| panic!("{language}: a failure message"))
            .to_owned();
        assert!(
            failure.starts_with(
                "outcome: the target answered `no-ticket`, the specification says `too-much`"
            ),
            "{language}: {failure}"
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
