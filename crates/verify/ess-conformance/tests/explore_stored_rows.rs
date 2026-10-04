//! The generated explorer draws commands whose answer depends on the stored row
//! (beyond10x/ess#221), and its text inputs reach `.count` boundaries and `example:` values
//! (beyond10x/ess#223), in both emitted languages.
//!
//! Before #221 a command with an `existing_instance`, `subject_state`, `state_change`,
//! `subject_field` or `subject_predicate` outcome was left out of every sequence. It is now
//! decided from the model's rows in the precedence order of
//! `docs/design/cross-record-and-stored-field-guards.md`: the input-guarded refusals, then
//! existence, then the held state and stored fields beside the accepting guards, then the
//! default. A `when_related` command reads a row of another entity and stays excluded by name.
//!
//! Before #223 a text input was drawn from `""`, `"a"`, `"b"` and the guards' text literals, so a
//! branch behind `secret.count < 12` was never passed and nothing it created was ever explored. A
//! text input now also draws its `example:`, and that example — or its own name, where it has
//! none — cut or cycled to n-1, n and n+1 characters for every `.count` literal compared with that
//! input; an Integer input also draws its `example:`. Another input of the same command keeps its
//! own pool.
//!
//! The fixture is `tests/fixtures/explore-stored-rows.yaml`; the targets are
//! `explore-stored-rows-target.mjs` and `explore_stored_rows_target.go`, switched by mode, run by
//! the preconditions drivers with the target swapped in. Every case runs in both lanes, and each
//! lane's result — failure, seed and shrunk trace included — must be the other's.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

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

const FIXTURE: &str = include_str!("fixtures/explore-stored-rows.yaml");

fn ir(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-stored-rows.yaml", text);
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-stored-rows.yaml"), raw)])
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

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("the fixture reads")
}

fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ess-explore-stored-rows-{}-{label}",
        std::process::id()
    ));
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

#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let text = std::fs::read_to_string(out.join(format!("{name}.json")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no result ({assert}):\n{log}"));
        results.insert(name.clone(), serde_json::from_str(&text).expect("JSON"));
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
    std::fs::copy(
        fixture("explore-stored-rows-target.mjs"),
        ts.join("explore-stored-rows-target.mjs"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        "./explore-stored-rows-target.mjs",
    );
    write(&ts.join("stored-rows-driver.mjs"), &driver);
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
        .args(["--test", "stored-rows-driver.mjs"])
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
        "module example.invalid/exploredraw\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_stored_rows_target.go"),
        module.join("essconform/explore_stored_rows_target_test.go"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        "newExploreRowsTarget(one.Mode)",
    );
    write(
        &module.join("essconform/explore_stored_rows_driver_test.go"),
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
            "--- PASS: TestExplorePreconditions/{}",
            case["name"].as_str().unwrap()
        );
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

const MODES: [&str; 9] = [
    "existing-ignored",
    "state-ignored",
    "changes-ignored",
    "field-ignored",
    "predicate-ignored",
    "count-inclusive",
    "uses-seven",
    "label-plain",
    "",
];

/// Every mode, run once per lane: `(typescript, go)`.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir(FIXTURE);
        let cases: Vec<Value> = MODES
            .iter()
            .map(|mode| {
                let name = if mode.is_empty() { "correct" } else { mode };
                json!({"name": name, "mode": mode, "allowExcluded": true})
            })
            .collect();
        let cases = Value::Array(cases);
        let root = scratch("lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn each_lane() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().0), ("go", &lanes().1)]
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("a list, not {value}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

/// The two lanes report the same result for `case`: the same reach, and for a failure the same
/// seed, message and shrunk trace.
fn paired(case: &str) -> &'static Value {
    assert_eq!(
        lanes().0.results[case],
        lanes().1.results[case],
        "typescript and go explore `{case}` differently"
    );
    &lanes().0.results[case]
}

/// The input of the last step of a failure's shrunk trace, and its command.
fn last_step(failure: &Value) -> (String, Value) {
    let trace = strings(&failure["trace"]);
    let line = trace
        .last()
        .unwrap_or_else(|| panic!("an empty trace: {failure}"));
    let (command, input) = line
        .split_once(' ')
        .unwrap_or_else(|| panic!("a trace line `{line}`"));
    (
        command.to_owned(),
        serde_json::from_str(input).unwrap_or_else(|_| panic!("JSON input in `{line}`")),
    )
}

const EVERY_INCLUDED_OUTCOME: [&str; 18] = [
    "exploredraw.keys.Bind/already-bound",
    "exploredraw.keys.Bind/bound",
    "exploredraw.keys.Bind/too-short",
    "exploredraw.keys.Enrol/enrolled",
    "exploredraw.keys.Resume/restated",
    "exploredraw.keys.Resume/resumed",
    "exploredraw.keys.Resume/stuck",
    "exploredraw.keys.Revoke/already-revoked",
    "exploredraw.keys.Revoke/revoked",
    "exploredraw.keys.Rotate/rotated",
    "exploredraw.keys.Rotate/same-secret",
    "exploredraw.keys.Rotate/unknown-key",
    "exploredraw.keys.Suspend/not-active",
    "exploredraw.keys.Suspend/suspended",
    "exploredraw.keys.Upgrade/kept",
    "exploredraw.keys.Upgrade/upgraded",
    "exploredraw.keys.Use/exhausted",
    "exploredraw.keys.Use/used",
];

#[test]
fn every_stored_row_command_is_drawn_and_every_branch_is_reached() {
    for (language, lane) in each_lane() {
        let found = &lane.results["correct"];
        assert_eq!(
            lane.asserts["correct"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        assert_eq!(
            strings(&found["reached"]),
            EVERY_INCLUDED_OUTCOME,
            "{language}: {found}"
        );
        assert!(
            found["unreached"].as_array().is_some_and(Vec::is_empty),
            "{language}: {found}"
        );
        // Only the command reading another entity's row is left out, and it says so by name.
        assert_eq!(
            found["excluded"],
            json!([{
                "subject": "exploredraw.keys.Grant",
                "reason": "outcome `no-owner` has a `related` condition"
            }]),
            "{language}: {found}"
        );
        assert_eq!(
            strings(&found["excludedOutcomes"]),
            [
                "exploredraw.keys.Grant/granted",
                "exploredraw.keys.Grant/no-owner"
            ],
            "{language}"
        );
        // The only draws left undecided name a key nobody holds, for a command that has no
        // unknown-instance branch.
        for draw in strings(&found["ambiguous"]) {
            assert!(
                draw.ends_with(": no record for the supplied instance"),
                "{language}: `{draw}` was redrawn\n{found}"
            );
        }
        assert!(
            found["undetermined"].as_array().is_some_and(Vec::is_empty),
            "{language}: {found}"
        );
    }
    paired("correct");
}

/// A target that answers one stored-row condition as if it were not declared is caught, with the
/// branch the model expected named in the failure, by the same seed and trace in both lanes.
#[test]
fn a_target_ignoring_a_stored_row_condition_is_caught() {
    for (mode, command, expected) in [
        ("existing-ignored", "exploredraw.keys.Bind", "already-bound"),
        (
            "state-ignored",
            "exploredraw.keys.Revoke",
            "already-revoked",
        ),
        ("changes-ignored", "exploredraw.keys.Resume", "restated"),
        ("field-ignored", "exploredraw.keys.Upgrade", "kept"),
        ("predicate-ignored", "exploredraw.keys.Use", "exhausted"),
    ] {
        for (language, lane) in each_lane() {
            assert!(
                lane.asserts[mode].starts_with("failed: "),
                "{language}: `{mode}` passed: {}\n{}",
                lane.asserts[mode],
                lane.results[mode]
            );
            let failure = &lane.results[mode]["failure"];
            let message = failure["message"].as_str().unwrap_or_default();
            assert!(
                message.contains(&format!("the specification says `{expected}`")),
                "{language}: {mode}: {failure}"
            );
            assert_eq!(
                last_step(failure).0,
                command,
                "{language}: {mode}: {failure}"
            );
        }
        paired(mode);
    }
}

/// A secret of exactly 12 characters — the example cut to the `.count` literal — is drawn, so a
/// target refusing it as too short is caught; the trace names the same 12-character secret in both
/// lanes. Which branch the model expected instead (`bound`, or `already-bound` for a key the trace
/// already bound) is the seed's, so only the refusal is asserted.
#[test]
fn a_text_input_reaches_its_count_boundary() {
    for (language, lane) in each_lane() {
        assert!(
            lane.asserts["count-inclusive"].starts_with("failed: "),
            "{language}: {}\n{}",
            lane.asserts["count-inclusive"],
            lane.results["count-inclusive"]
        );
    }
    let failure = &paired("count-inclusive")["failure"];
    let (command, input) = last_step(failure);
    assert_eq!(command, "exploredraw.keys.Bind", "{failure}");
    assert_eq!(input["secret"], "correct-hors", "{failure}");
    assert!(
        failure["message"]
            .as_str()
            .is_some_and(|it| it.contains("the target answered `too-short`")),
        "{failure}"
    );
}

/// The Integer input's `example:` is drawn: a target refusing `uses: 7` is caught on it.
#[test]
fn an_integer_input_draws_its_example() {
    for (language, lane) in each_lane() {
        assert!(
            lane.asserts["uses-seven"].starts_with("failed: "),
            "{language}: {}\n{}",
            lane.asserts["uses-seven"],
            lane.results["uses-seven"]
        );
    }
    let failure = &paired("uses-seven")["failure"];
    let (command, input) = last_step(failure);
    assert_eq!(command, "exploredraw.keys.Bind", "{failure}");
    assert_eq!(input["uses"], 7, "{failure}");
}

/// `label` carries no `example:` and is compared with nothing, so it is drawn from the plain pool:
/// the secret's example and boundary strings are `secret`'s, not every text input's.
#[test]
fn a_text_input_compared_with_nothing_keeps_the_plain_pool() {
    for (language, lane) in each_lane() {
        assert_eq!(
            lane.asserts["label-plain"], "ok",
            "{language}: {}",
            lane.results["label-plain"]
        );
        assert!(
            strings(&lane.results["label-plain"]["reached"])
                .contains(&"exploredraw.keys.Bind/bound"),
            "{language}: {}",
            lane.results["label-plain"]
        );
    }
    paired("label-plain");
}

fn replace(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The fixture with `Annotate`, whose refusal reads a stored field `note` no command sets.
fn unset_field() -> String {
    let text = replace(
        FIXTURE,
        "      - {name: uses, type: Integer}\n    lifecycle:",
        "      - {name: uses, type: Integer}\n      - {name: note, type: String}\n    lifecycle:",
    );
    let text = replace(
        &text,
        "      - exploredraw.keys.Grant\n\nerrors:",
        "      - exploredraw.keys.Grant\n      - exploredraw.keys.Annotate\n\nerrors:",
    );
    replace(
        &text,
        "\nviews:",
        "
  - name: exploredraw.keys.Annotate
    input:
      - {name: key_id, type: exploredraw.keys.KeyId}
    outcomes:
      - name: noted
        when_subject:
          predicate: note == \"x\"
        error: exploredraw.keys.Stuck
      - name: annotated
        updates: exploredraw.keys.Key
        instance: key_id
        emits: [exploredraw.keys.Kept]
        payload: {exploredraw.keys.Kept: {key_id: input.key_id}}

views:",
    )
}

/// A predicate over a stored field no command sets is not decided on a guess: the draw is
/// reported in `undetermined` and redrawn, in both lanes alike, and nothing else changes.
#[test]
fn a_stored_field_no_command_sets_is_undetermined_and_redrawn() {
    let ir = ir(&unset_field());
    let cases = json!([{"name": "correct", "mode": "", "allowExcluded": true}]);
    let root = scratch("unset");
    let lanes = [
        ("typescript", typescript(&root, &ir, &cases)),
        ("go", go(&root, &ir, &cases)),
    ];
    std::fs::remove_dir_all(&root).ok();
    for (language, lane) in &lanes {
        let found = &lane.results["correct"];
        assert_eq!(
            strings(&found["undetermined"]),
            ["exploredraw.keys.Annotate guard of `noted` reads note, which no command set"],
            "{language}: {found}"
        );
        assert_eq!(
            strings(&found["unreached"]),
            [
                "exploredraw.keys.Annotate/annotated",
                "exploredraw.keys.Annotate/noted"
            ],
            "{language}: {found}"
        );
        assert!(found.get("failure").is_none(), "{language}: {found}");
        for outcome in EVERY_INCLUDED_OUTCOME {
            assert!(
                strings(&found["reached"]).contains(&outcome),
                "{language}: `{outcome}` unreached: {found}"
            );
        }
    }
    assert_eq!(
        lanes[0].1.results["correct"], lanes[1].1.results["correct"],
        "typescript and go explore the unset field differently"
    );
}

/// The fixture with an accepting `bulk` branch on `Use` declared *before* the stored-row refusal
/// `exhausted`.
fn bulk_declared_first() -> String {
    replace(
        FIXTURE,
        "      - name: exhausted\n",
        "      - name: bulk
        when: amount > 100
        updates: exploredraw.keys.Key
        instance: key_id
        emits: [exploredraw.keys.Used]
        payload: {exploredraw.keys.Used: {key_id: input.key_id}}
      - name: exhausted\n",
    )
}

/// Where an accepting guard declared before the stored-row branch that holds holds as well, the
/// interpreter (declaration order) answers `bulk` and the precedence order `exhausted`: that draw
/// alone is redrawn and reported, in both lanes alike, and every other draw of `Use` is decided.
#[test]
fn an_accepting_guard_declared_before_the_held_row_is_the_one_undecided_draw() {
    let ir = ir(&bulk_declared_first());
    let cases = json!([{"name": "bulk", "mode": "bulk", "allowExcluded": true}]);
    let root = scratch("bulk-first-declared");
    let lanes = [
        ("typescript", typescript(&root, &ir, &cases)),
        ("go", go(&root, &ir, &cases)),
    ];
    std::fs::remove_dir_all(&root).ok();
    for (language, lane) in &lanes {
        let found = &lane.results["bulk"];
        assert_eq!(lane.asserts["bulk"], "ok", "{language}: {found}");
        assert!(
            strings(&found["ambiguous"]).contains(&"exploredraw.keys.Use: bulk, exhausted"),
            "{language}: {found}"
        );
        for outcome in [
            "exploredraw.keys.Use/bulk",
            "exploredraw.keys.Use/exhausted",
            "exploredraw.keys.Use/used",
        ] {
            assert!(
                strings(&found["reached"]).contains(&outcome),
                "{language}: `{outcome}` unreached: {found}"
            );
        }
    }
    assert_eq!(
        lanes[0].1.results["bulk"], lanes[1].1.results["bulk"],
        "typescript and go explore the declared-first overlap differently"
    );
}
