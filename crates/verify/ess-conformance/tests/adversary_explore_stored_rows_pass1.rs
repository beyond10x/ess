//! Adversary pass 1 against beyond10x/ess#221 and #223 (the explorer draws stored-row conditions
//! and `.count` boundaries). Each case drives both emitted explorers over a variant of
//! `tests/fixtures/explore-stored-rows.yaml` with the specification's own target
//! (`explore-stored-rows-target.mjs`, `explore_stored_rows_target.go`, mode `""`), so every
//! answer the target gives is the one the specification names.
//!
//! 1. A stored field holding `null` (an `Optional` field set from an absent `Optional` input) makes
//!    a `when_subject` predicate `Unknown`. That is a fact of one stored row, which another record
//!    need not share, so the draw is to be redrawn and reported as `undetermined` — as the unit
//!    does for a field no command set. The explorers instead blame the generated input and exclude
//!    the whole command.
//! 2. A stored-row branch declared *after* the branch that answers is never read by the model
//!    interpreter (`interpret::execute::select` stops at the first branch that holds). The
//!    explorers read every stored-row branch first, so a later undecidable one discards a draw the
//!    specification decides, and the branch that answers it is never reached.
//!
//! Two controls run the model interpreter over the same requests (it decides both), and a third
//! case checks that a non-ASCII `example:` is cut alike in both lanes (green when written).
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

mod support_scratch;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn scratch(label: &str) -> support_scratch::Scratch {
    let root = support_scratch::Scratch::adopt(std::env::temp_dir().join(format!(
        "ess-adversary-stored-rows-{}-{label}",
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

struct Lane {
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
}

fn read_lane(out: &Path, cases: &Value, log: &str) -> Lane {
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
    Lane { results, asserts }
}

/// The specification's target, extended for the adversary's variants of the fixture:
///
/// * `Annotate`: `spent` for a key whose `uses` is at least 0, as the specification says. Any
///   other `Annotate` reads `note`, which nothing set, and is unsupported.
/// * mode `bulk`: `Use` of more than 100 answers `bulk` unless the key is exhausted, as the
///   specification with a `bulk` branch says. Mode `bulk-first` answers `bulk` for every amount
///   over 100, exhausted or not: the stored-row refusal is checked after the accepting guard.
const ANNOTATE_TS: &str = r"import { newTarget as rows } from './explore-stored-rows-target.mjs';
import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  const bulk = mode.startsWith('bulk');
  const inner = rows(bulk ? '' : mode);
  let writes = 0;
  return {
    ...inner,
    executeCommand(request) {
      const key = inner
        .queryView({ view: 'exploredraw.keys.Keys' })
        .rows.find((row) => row.key_id === request.input.key_id);
      if (request.command === 'exploredraw.keys.Annotate') {
        if (key !== undefined && Number(key.uses) >= 0) {
          return { outcome: 'spent', error: 'exploredraw.keys.Exhausted', directEvents: [] };
        }
        throw unsupported('Annotate of a key with negative uses reads a note nothing set');
      }
      if (bulk && request.command === 'exploredraw.keys.Use' && key !== undefined) {
        const amount = Number(request.input.amount);
        const exhausted = Number(key.uses) < amount;
        if (amount > 100 && (mode === 'bulk-first' || !exhausted)) {
          writes += 1;
          return {
            outcome: 'bulk',
            consistency: `bulk${writes}`,
            directEvents: [{ event: 'exploredraw.keys.Used', payload: { key_id: key.key_id } }],
          };
        }
      }
      return inner.executeCommand(request);
    },
  };
}
";

const ANNOTATE_GO: &str = r#"package essconform

import "strings"

type advAnnotateTarget struct {
	*exploreRowsTarget
	advMode string
}

func newAdvAnnotateTarget(mode string) *advAnnotateTarget {
	inner := mode
	if strings.HasPrefix(mode, "bulk") {
		inner = ""
	}
	return &advAnnotateTarget{newExploreRowsTarget(inner), mode}
}

func (t *advAnnotateTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	key := t.find(request.Input["key_id"])
	if request.Command == "exploredraw.keys.Annotate" {
		if key != nil && exploreRowsNumber(key.uses) >= 0 {
			return exploreRowsRefused("spent", "exploredraw.keys.Exhausted"), nil
		}
		return CommandResult{}, exploreRowsUnsupported{"Annotate of a key with negative uses reads a note nothing set"}
	}
	if strings.HasPrefix(t.advMode, "bulk") && request.Command == "exploredraw.keys.Use" && key != nil {
		amount := exploreRowsNumber(request.Input["amount"])
		exhausted := exploreRowsNumber(key.uses) < amount
		if amount > 100 && (t.advMode == "bulk-first" || !exhausted) {
			return t.answered("bulk", "exploredraw.keys.Used", "key_id", key.id), nil
		}
	}
	return t.exploreRowsTarget.ExecuteCommand(request)
}
"#;

fn typescript(root: &Path, ir: &EssIr, cases: &Value, annotate: bool) -> Lane {
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
    write(&ts.join("adversary-annotate-target.mjs"), ANNOTATE_TS);
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        if annotate {
            "./adversary-annotate-target.mjs"
        } else {
            "./explore-stored-rows-target.mjs"
        },
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
        .expect("`tsc` is on PATH");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "stored-rows-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, &log)
}

fn go(root: &Path, ir: &EssIr, cases: &Value, annotate: bool) -> Lane {
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
    write(
        &module.join("essconform/adversary_annotate_target_test.go"),
        ANNOTATE_GO,
    );
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        if annotate {
            "newAdvAnnotateTarget(one.Mode)"
        } else {
            "newExploreRowsTarget(one.Mode)"
        },
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
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, &log)
}

fn replace(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("a list, not {value}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

/// Both lanes, the correct target, over `text`.
fn explore_correct(label: &str, text: &str, annotate: bool) -> [(&'static str, Value, String); 2] {
    let ir = ir(text);
    let cases = json!([{"name": "correct", "mode": "", "allowExcluded": true}]);
    let root = scratch(label);
    let ts = typescript(&root, &ir, &cases, annotate);
    let go = go(&root, &ir, &cases, annotate);
    std::fs::remove_dir_all(&root).ok();
    [
        (
            "typescript",
            ts.results["correct"].clone(),
            ts.asserts["correct"].clone(),
        ),
        (
            "go",
            go.results["correct"].clone(),
            go.asserts["correct"].clone(),
        ),
    ]
}

/// The fixture with `uses` an `Optional<Integer>` on the key, on `Bind`'s input and on the view:
/// a key bound without `uses` stores `null`, and `Use`'s `exhausted` (`uses < input.amount`) is
/// `Unknown` over that one row.
fn optional_uses() -> String {
    let text = replace(
        FIXTURE,
        "      - {name: uses, type: Integer}\n",
        "      - {name: uses, type: Optional<Integer>}\n",
    );
    replace(
        &text,
        "      - {name: uses, type: Integer, example: 7}\n",
        "      - {name: uses, type: Optional<Integer>, example: 7}\n",
    )
}

/// A `null` stored field is a fact of one row: the draw over it is redrawn and reported, and the
/// command stays in exploration, its two branches reached over the keys bound with `uses`.
#[test]
fn adversary_a_null_stored_field_is_not_blamed_on_the_generated_input() {
    let mut wrong = Vec::new();
    for (language, found, assert) in explore_correct("null-field", &optional_uses(), false) {
        let excluded = found["excluded"].clone();
        let reached = strings(&found["reached"])
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if excluded
            != json!([{
                "subject": "exploredraw.keys.Grant",
                "reason": "outcome `no-owner` has a `related` condition"
            }])
            || !reached
                .iter()
                .any(|it| it == "exploredraw.keys.Use/exhausted")
            || !reached.iter().any(|it| it == "exploredraw.keys.Use/used")
        {
            wrong.push(format!(
                "{language} ({assert}): excluded {excluded}; undetermined {}",
                found["undetermined"]
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "`Use` is excluded over one row's null `uses`:\n{}",
        wrong.join("\n")
    );
}

/// The fixture with `Annotate`, which refuses a key whose `uses` is at least 0 (`spent`, `when_subject`) before
/// it reads a stored field `note` no command sets (`noted`, `when_subject`).
fn answered_before_unset_field() -> String {
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
      - name: spent
        when_subject:
          predicate: uses >= 0
        error: exploredraw.keys.Exhausted
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

/// A key with `uses >= 0` is answered by `spent`, the first branch that holds; `noted`, after it, is never
/// read (`interpret::execute::select`). So `Annotate` of such a key is decided, and `spent` is
/// reached whenever such a key is annotated.
#[test]
fn adversary_a_stored_row_branch_after_the_answering_one_is_not_read() {
    let mut wrong = Vec::new();
    for (language, found, assert) in
        explore_correct("after-answer", &answered_before_unset_field(), true)
    {
        if !strings(&found["reached"]).contains(&"exploredraw.keys.Annotate/spent") {
            wrong.push(format!(
                "{language} ({assert}): unreached {}; undetermined {}",
                found["unreached"], found["undetermined"]
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "`Annotate/spent` is never reached:\n{}",
        wrong.join("\n")
    );
}

/// What the model interpreter answers to each request in turn, from an empty store: the branch
/// taken, or why it determines none.
fn interpret(text: &str, requests: &[(&str, Value)]) -> Vec<Result<String, String>> {
    use ess_conformance::interpret::execute::{self, Externals, Store};
    use ess_primitives::node::Node;
    let ir = ir(text);
    let mut store = Store::default();
    let mut answers = Vec::new();
    for (command, input) in requests {
        let input: BTreeMap<String, Node> =
            serde_json::from_value(input.clone()).expect("an input the model reads");
        let command = format!("exploredraw.keys.{command}")
            .parse()
            .expect("a name");
        match execute::execute(&ir, &store, &command, &input, &Externals::Withheld) {
            Ok(steps) => {
                assert_eq!(steps.len(), 1, "{steps:?}");
                let step = steps.into_iter().next().expect("one step");
                answers.push(Ok(step
                    .outcome
                    .map(|outcome| outcome.outcome.to_string())
                    .unwrap_or_default()));
                store = step.next;
            }
            Err(why) => answers.push(Err(why.to_string())),
        }
    }
    answers
}

/// The oracle for the first case: `Use` over the row with `null` uses is undecidable for that
/// request alone; the same command over a row holding `uses` is decided.
#[test]
fn adversary_control_the_interpreter_decides_use_per_row() {
    let secret = "correct-horse-battery";
    let answers = interpret(
        &optional_uses(),
        &[
            (
                "Bind",
                json!({"key_id": "k1", "secret": secret, "label": "a", "uses": null}),
            ),
            ("Use", json!({"key_id": "k1", "amount": 1})),
            (
                "Bind",
                json!({"key_id": "k2", "secret": secret, "label": "a", "uses": 7}),
            ),
            ("Use", json!({"key_id": "k2", "amount": 1})),
            ("Use", json!({"key_id": "k2", "amount": 9})),
        ],
    );
    assert_eq!(answers[0], Ok("bound".to_owned()), "{answers:?}");
    assert!(
        answers[1]
            .as_ref()
            .is_err_and(|why| why.contains("is Unknown over this input")),
        "{answers:?}"
    );
    assert_eq!(
        answers[2..],
        [
            Ok("bound".to_owned()),
            Ok("used".to_owned()),
            Ok("exhausted".to_owned())
        ],
        "{answers:?}"
    );
}

/// The oracle for the second case: `Annotate` of a key holding `uses: 7` is `spent`, though
/// `noted`, declared after it, reads a field nothing set.
#[test]
fn adversary_control_the_interpreter_answers_before_the_unset_field() {
    let answers = interpret(
        &answered_before_unset_field(),
        &[
            (
                "Bind",
                json!({"key_id": "k1", "secret": "correct-horse-battery", "label": "a", "uses": 7}),
            ),
            ("Annotate", json!({"key_id": "k1"})),
        ],
    );
    assert_eq!(
        answers,
        [Ok("bound".to_owned()), Ok("spent".to_owned())],
        "{answers:?}"
    );
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

/// An `example:` outside the Basic Multilingual Plane and outside ASCII is cut at the `.count`
/// boundary by Unicode scalar values in both lanes: one character for the key, four for the
/// Cyrillic, so the 12-character secret is the same in Go (runes) and TypeScript (where the key is
/// two UTF-16 units), and both lanes report the same seed, message and shrunk trace.
#[test]
fn adversary_a_non_ascii_example_is_cut_alike_in_both_lanes() {
    let text = replace(
        FIXTURE,
        "example: \"correct-horse-battery\"",
        "example: \"\u{1F511}\u{43A}\u{43B}\u{44E}\u{447}-horse-battery\"",
    );
    let ir = ir(&text);
    let cases =
        json!([{"name": "count-inclusive", "mode": "count-inclusive", "allowExcluded": true}]);
    let root = scratch("non-ascii");
    let ts = typescript(&root, &ir, &cases, false);
    let go = go(&root, &ir, &cases, false);
    std::fs::remove_dir_all(&root).ok();
    for (language, lane) in [("typescript", &ts), ("go", &go)] {
        assert!(
            lane.asserts["count-inclusive"].starts_with("failed: "),
            "{language}: {}",
            lane.results["count-inclusive"]
        );
    }
    assert_eq!(
        ts.results["count-inclusive"], go.results["count-inclusive"],
        "typescript and go cut the non-ASCII example differently"
    );
    let failure = &ts.results["count-inclusive"]["failure"];
    let (command, input) = last_step(failure);
    assert_eq!(command, "exploredraw.keys.Bind", "{failure}");
    assert_eq!(
        input["secret"], "\u{1F511}\u{43A}\u{43B}\u{44E}\u{447}-horse-",
        "{failure}"
    );
}

/// The fixture with a `bulk` branch on `Use`: an accepting `when: amount > 100` declared after
/// the stored-row refusal `exhausted` (`uses < input.amount`).
fn bulk_use() -> String {
    replace(
        FIXTURE,
        "        error: exploredraw.keys.Exhausted\n      - name: used\n",
        "        error: exploredraw.keys.Exhausted
      - name: bulk
        when: amount > 100
        updates: exploredraw.keys.Key
        instance: key_id
        emits: [exploredraw.keys.Used]
        payload: {exploredraw.keys.Used: {key_id: input.key_id}}
      - name: used\n",
    )
}

/// The oracle: a key holding `uses: 7` asked for 101 is `exhausted` — the held row before the
/// accepting guard, as the precedence order (`cross-record-and-stored-field-guards.md`, steps 4
/// and 6) and the interpreter both say.
#[test]
fn adversary_control_the_interpreter_refuses_exhausted_before_bulk() {
    let answers = interpret(
        &bulk_use(),
        &[
            (
                "Bind",
                json!({"key_id": "k1", "secret": "correct-horse-battery", "label": "a", "uses": 7}),
            ),
            ("Use", json!({"key_id": "k1", "amount": 101})),
            ("Use", json!({"key_id": "k1", "amount": 5})),
            (
                "Bind",
                json!({"key_id": "k2", "secret": "correct-horse-battery", "label": "a", "uses": 500}),
            ),
            ("Use", json!({"key_id": "k2", "amount": 101})),
        ],
    );
    assert_eq!(
        answers,
        [
            Ok("bound".to_owned()),
            Ok("exhausted".to_owned()),
            Ok("used".to_owned()),
            Ok("bound".to_owned()),
            Ok("bulk".to_owned()),
        ],
        "{answers:?}"
    );
}

/// A target that checks the accepting guard before the stored-row refusal answers `bulk` where
/// the specification says `exhausted`. Every request on which the two differ is one where both
/// hold, which the explorer marks ambiguous and redraws, so the target passes. The specification's
/// own target (mode `bulk`) is the control and must pass.
#[test]
fn adversary_a_target_checking_an_accepting_guard_before_the_held_row_is_caught() {
    let ir = ir(&bulk_use());
    let cases = json!([
        {"name": "bulk", "mode": "bulk", "allowExcluded": true},
        {"name": "bulk-first", "mode": "bulk-first", "allowExcluded": true}
    ]);
    let root = scratch("bulk");
    let ts = typescript(&root, &ir, &cases, true);
    let go = go(&root, &ir, &cases, true);
    std::fs::remove_dir_all(&root).ok();
    let mut wrong = Vec::new();
    for (language, lane) in [("typescript", &ts), ("go", &go)] {
        assert_eq!(
            lane.asserts["bulk"], "ok",
            "{language}: the control target fails: {}",
            lane.results["bulk"]
        );
        if !lane.asserts["bulk-first"].starts_with("failed: ") {
            wrong.push(format!(
                "{language} ({}): ambiguous {}",
                lane.asserts["bulk-first"], lane.results["bulk-first"]["ambiguous"]
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "a target answering `bulk` over an exhausted key passes:\n{}",
        wrong.join("\n")
    );
}
