//! Adversary pass 1 on beyond10x/ess#538: the explorers' model of a selected-rows effect
//! (`instances: {where: …}`) held to the interpreter on the same model.
//!
//! The fixture is `tests/fixtures/adversary-l3-selected-rows.yaml`; its targets
//! (`adversary-l3-selected-rows-target.mjs`, `adversary_l3_selected_rows_target.go`) answer as the
//! interpreter does, and their `closes-untagged` mode also closes untagged seats. Each explorer case
//! is paired with one showing the interpreter decide the same step.
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
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::target::*;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/adversary-l3-selected-rows.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("adversary-l3-selected-rows.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("adversary-l3-selected-rows.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

// ---- the interpreter, the reference ----------------------------------------------------------

fn context() -> ScenarioContext {
    ScenarioContext::new(
        "advrows.desk/authored/adversary".parse().unwrap(),
        CorrelationId::new("adversary").unwrap(),
    )
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn invoke(target: &Interpreted, command: &str, input: &[(&str, Node)]) -> SemanticCommandResult {
    target
        .execute_command(SemanticCommandRequest {
            command: format!("advrows.desk.{command}").parse().unwrap(),
            actor: None,
            caller: None,
            input: input
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
            correlation: context().correlation,
        })
        .unwrap_or_else(|error| panic!("the interpreter answers `{command}`: {error}"))
}

fn outcome(result: &SemanticCommandResult) -> String {
    result
        .outcome
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn created(result: &SemanticCommandResult, field: &str) -> Node {
    result.direct_events[0].payload[field].clone()
}

fn states(target: &Interpreted, view: &str) -> Vec<Node> {
    target
        .query_view(SemanticViewRequest {
            view: format!("advrows.desk.{view}").parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
        .iter()
        .map(|row| row["state"].clone())
        .collect()
}

fn interpreter() -> Interpreted {
    let target = Interpreted::for_model(ir());
    target.begin_scenario(&context()).unwrap();
    target
}

// ---- the explorers ---------------------------------------------------------------------------

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

/// One lane: each case's result and assertion, and the runner log.
#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
    log: String,
}

const CASE_NAMES: [&str; 2] = ["correct", "closes-untagged"];
const CASES: &str =
    r#"[{"name": "correct"}, {"name": "closes-untagged", "mode": "closes-untagged"}]"#;

fn read_lane(out: &Path, names: &[&str], log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for name in names {
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let text = std::fs::read_to_string(out.join(format!("{name}.json")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no result ({assert}):\n{log}"));
        results.insert(
            (*name).to_owned(),
            serde_json::from_str(&text).expect("JSON"),
        );
        asserts.insert((*name).to_owned(), assert.trim_end().to_owned());
    }
    Lane {
        results,
        asserts,
        log,
    }
}

/// Which fixture target a lane runs, and the cases it runs it with.
struct Under {
    mjs: &'static str,
    go: &'static str,
    constructor: &'static str,
    cases: &'static str,
    names: &'static [&'static str],
}

const ROWS: Under = Under {
    mjs: "adversary-l3-selected-rows-target.mjs",
    go: "adversary_l3_selected_rows_target.go",
    constructor: "newAdvL3Target",
    cases: CASES,
    names: &CASE_NAMES,
};

fn typescript(root: &Path, ir: &EssIr, under: &Under) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    let target = under.mjs;
    std::fs::copy(fixture(target), ts.join(target)).expect("the target copies");
    let driver = std::fs::read_to_string(fixture("explore-preconditions-driver.mjs"))
        .unwrap()
        .replace("./explore-preconditions-target.mjs", &format!("./{target}"));
    write(&ts.join("adversary-driver.mjs"), &driver);
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
        .args(["--test", "adversary-driver.mjs"])
        .env("ESS_EXPLORE_CASES", under.cases)
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for (index, name) in under.names.iter().enumerate() {
        let line = format!("ok {} - {name}", index + 1);
        assert!(log.lines().any(|it| it == line), "`{line}` missing:\n{log}");
    }
    read_lane(&out, under.names, log)
}

fn go(root: &Path, ir: &EssIr, under: &Under) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/adversaryl3\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture(under.go),
        module.join(format!(
            "essconform/{}_test.go",
            under.go.trim_end_matches(".go")
        )),
    )
    .expect("the target copies");
    let driver = std::fs::read_to_string(fixture("explore_preconditions_driver_test.go"))
        .unwrap()
        .replace(
            "newExplorePreTarget(one.Mode)",
            &format!("{}(one.Mode)", under.constructor),
        );
    write(
        &module.join("essconform/adversary_l3_driver_test.go"),
        &driver,
    );
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExplorePreconditions",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", under.cases)
        .env("ESS_EXPLORE_OUT", &out)
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for name in under.names {
        let line = format!("--- PASS: TestExplorePreconditions/{name}");
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` missing:\n{log}"
        );
    }
    read_lane(&out, under.names, log)
}

fn lanes() -> &'static [(&'static str, Lane); 2] {
    static LANES: OnceLock<[(&'static str, Lane); 2]> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let root =
            support_scratch::Scratch::new(format!("ess-adversary-l3-{}", std::process::id()));
        [
            ("typescript", typescript(&root, &ir, &ROWS)),
            ("go", go(&root, &ir, &ROWS)),
        ]
    })
}

/// The `correct` case passed in `lane`; its `undetermined` reasons naming `needle`.
fn undetermined_about(language: &str, lane: &Lane, needle: &str) -> Vec<String> {
    let found = &lane.results["correct"];
    assert_eq!(
        lane.asserts["correct"], "ok",
        "{language}: {found}\n{}",
        lane.log
    );
    found["undetermined"]
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(Value::as_str)
        .filter(|reason| reason.contains(needle))
        .map(str::to_owned)
        .collect()
}

// ---- an `Optional<…>` field no creation writes --------------------------------------------------

/// The interpreter reads an `Optional<…>` field nothing wrote as absent: `tag == input.team` does
/// not hold over an untagged seat, which is not selected, and the step is decided (`set_effects.rs`,
/// `select` and `only_absent`).
#[test]
fn the_interpreter_selects_no_untagged_seat() {
    let target = interpreter();
    let tagged = created(
        &invoke(&target, "Start", &[("team", text("red"))]),
        "seat_id",
    );
    let _untagged = created(
        &invoke(&target, "Start", &[("team", text("red"))]),
        "seat_id",
    );
    let tag = invoke(&target, "Tag", &[("seat_id", tagged), ("tag", text("red"))]);
    assert_eq!(outcome(&tag), "advrows.desk.Tag/tagged");
    let closed = invoke(&target, "CloseTagged", &[("team", text("red"))]);
    assert_eq!(outcome(&closed), "advrows.desk.CloseTagged/closed");
    let held = states(&target, "Seats");
    assert_eq!(held.len(), 2, "{held:?}");
    assert!(
        held.contains(&text("Closed")) && held.contains(&text("Open")),
        "{held:?}"
    );
}

/// Both explorers, on the same model, against a target answering as the interpreter does. The
/// explorer's doc comment (`exploreSelectedRows`, `selectedRows`) promises the interpreter's
/// reading: "One left Unknown by a stored field held as null (an `Optional` holding nothing) …
/// does not hold, so the row is not selected". But the model holds an `Optional` field a creation
/// did not write as *missing*, not null, so every `CloseTagged` drawn while an untagged seat exists
/// is reported "reads tag, which no command set" (though `Tag` sets it) and redrawn.
#[test]
fn the_explorers_select_no_untagged_seat_as_the_interpreter_does() {
    let mut wrong = Vec::new();
    for (language, lane) in lanes() {
        let about = undetermined_about(language, lane, "reads tag");
        if !about.is_empty() {
            wrong.push(format!("{language}: {about:?}"));
        }
    }
    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "the interpreter decides `CloseTagged` beside an untagged seat; the explorers redraw it"
    );
}

/// What that costs: `CloseTagged` is never executed while an untagged seat exists, so a target
/// that also closes untagged seats — which the interpreter would not — passes exploration.
#[test]
fn the_explorers_catch_a_target_closing_untagged_seats() {
    let mut passed = Vec::new();
    for (language, lane) in lanes() {
        let assert = &lane.asserts["closes-untagged"];
        if !assert.starts_with("failed: explore: seed ") {
            passed.push(format!(
                "{language}: {assert} {}",
                lane.results["closes-untagged"]
            ));
        }
    }
    assert_eq!(
        passed,
        Vec::<String>::new(),
        "a target closing untagged seats passes exploration"
    );
}

// ---- a row outside the move's `from` states ---------------------------------------------------

/// The interpreter skips a row resting outside the move's `from` states before it reads the
/// filter (`set_effects.rs`, `select`): an `Idle` desk, whose `zone` nothing has written yet, is
/// never asked about it, and `Finish` is decided.
#[test]
fn the_interpreter_never_reads_the_filter_of_a_row_outside_the_move() {
    let target = interpreter();
    let active = created(
        &invoke(&target, "OpenDesk", &[("owner", text("red"))]),
        "desk_id",
    );
    let activate = invoke(
        &target,
        "Activate",
        &[("desk_id", active), ("zone", text("red"))],
    );
    assert_eq!(outcome(&activate), "advrows.desk.Activate/activated");
    let _idle = created(
        &invoke(&target, "OpenDesk", &[("owner", text("red"))]),
        "desk_id",
    );
    let finished = invoke(&target, "Finish", &[("zone", text("red"))]);
    assert_eq!(outcome(&finished), "advrows.desk.Finish/finished");
    let held = states(&target, "Desks");
    assert_eq!(held.len(), 2, "{held:?}");
    assert!(
        held.contains(&text("Done")) && held.contains(&text("Idle")),
        "{held:?}"
    );
}

/// Both explorers evaluate the filter over every row of the entity, `from` state or not, so an
/// `Idle` desk's unwritten `zone` makes every `Finish` drawn beside it undetermined, where the
/// interpreter decides it.
#[test]
fn the_explorers_skip_rows_outside_the_move_before_reading_the_filter() {
    let mut wrong = Vec::new();
    for (language, lane) in lanes() {
        let about = undetermined_about(language, lane, "reads zone");
        if !about.is_empty() {
            wrong.push(format!("{language}: {about:?}"));
        }
    }
    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "the interpreter decides `Finish` beside an idle desk; the explorers redraw it"
    );
}

// ---- an `affects:` entry beside a supplied subject ----------------------------------------------

const AFFECTS: &str = include_str!("fixtures/adversary-l3-affects.yaml");

fn affects_ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("adversary-l3-affects.yaml", AFFECTS);
    let raw = RawSpecFile::parse(AFFECTS).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("adversary-l3-affects.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

const AFFECTED: Under = Under {
    mjs: "adversary-l3-affects-target.mjs",
    go: "adversary_l3_affects_target.go",
    constructor: "newAdvL3AffTarget",
    cases: r#"[{"name": "correct", "allowExcluded": true}]"#,
    names: &["correct"],
};

/// The interpreter applies `Note`'s `affects:` entry: the other seat of the team comes to hold the
/// reason too (`set_effects.rs`, `apply`).
#[test]
fn the_interpreter_writes_every_seat_an_affects_entry_selects() {
    let target = Interpreted::for_model(affects_ir());
    target.begin_scenario(&context()).unwrap();
    let start = |team: &str| {
        target
            .execute_command(SemanticCommandRequest {
                command: "advaff.desk.Start".parse().unwrap(),
                actor: None,
                caller: None,
                input: BTreeMap::from([("team".to_owned(), text(team))]),
                correlation: context().correlation,
            })
            .unwrap()
            .direct_events[0]
            .payload["seat_id"]
            .clone()
    };
    let noted = start("red");
    let _other = start("red");
    let result = target
        .execute_command(SemanticCommandRequest {
            command: "advaff.desk.Note".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([
                ("seat_id".to_owned(), noted),
                ("reason".to_owned(), text("hungup")),
            ]),
            correlation: context().correlation,
        })
        .unwrap();
    assert_eq!(outcome(&result), "advaff.desk.Note/noted");
    let reasons: Vec<Node> = target
        .query_view(SemanticViewRequest {
            view: "advaff.desk.Seats".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
        .iter()
        .map(|row| row["last_reason"].clone())
        .collect();
    assert_eq!(reasons, [text("hungup"), text("hungup")]);
}

/// Neither explorer models `affects:`. Before this pass neither excluded it either, so `Note` was
/// drawn, modelled as writing its subject alone, and a target answering as the interpreter does was
/// reported as disagreeing on the other seat's `last_reason`. `exploreEffectRefusal` /
/// `effectRefusal` now leave such an outcome out by name, as a `deletes:` is: the correct target
/// passes once exclusions are allowed, and `Note` is excluded with the `affects:` reason.
#[test]
fn the_explorers_pass_a_target_applying_an_affects_entry() {
    let ir = affects_ir();
    let root =
        support_scratch::Scratch::new(format!("ess-adversary-l3-aff-{}", std::process::id()));
    let mut failed = Vec::new();
    for (language, lane) in [
        ("typescript", typescript(&root, &ir, &AFFECTED)),
        ("go", go(&root, &ir, &AFFECTED)),
    ] {
        if lane.asserts["correct"] != "ok" {
            failed.push(format!(
                "{language}: {} {}",
                lane.asserts["correct"], lane.results["correct"]["failure"]
            ));
        }
        let excluded = &lane.results["correct"]["excluded"];
        if *excluded
            != serde_json::json!([{
                "subject": "advaff.desk.Note",
                "reason": "outcome `noted` writes the rows its `affects:` selects, which the model \
                           does not apply"
            }])
        {
            failed.push(format!("{language}: excluded {excluded}"));
        }
    }
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "a target applying `affects:` as the interpreter does fails exploration"
    );
}
