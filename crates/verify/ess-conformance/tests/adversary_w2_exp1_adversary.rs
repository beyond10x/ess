//! Adversary pass 1 for ESS wave W2 unit EXP1.
//!
//! 1. `story:component-explorer-walks-only-its-component` scopes by `accepts` *or* the domains a
//!    component `owns`, and keeps the views of the domains it owns. `tests/explore_component.rs`
//!    exercises `accepts` alone over a model with no view, so the `owns` half of `exploreScope`
//!    and its view filter are read by no test. Here two components each own one domain and list no
//!    command: the shelf component's package must explore `PutItem` and the shelf view, and must
//!    leave the memo command and view out. `ESS_ADVERSARY_MUTANT` patches the emitted
//!    `explore.go` (a copy in scratch, never the tree) to show each half is now held.
//!
//! A missing `go` panics rather than skipping.

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

const SYSTEM: &str = r"format: ess/29
system: exploreown
version: v1
summary: Two domains, each owned by one component that lists no commands.
domains:
  - exploreown.shelf
  - exploreown.memo
components:
  - component: shelf-service
    owns: {domains: [exploreown.shelf]}
  - component: memo-service
    owns: {domains: [exploreown.memo]}
";

const SHELF: &str = r"domain: exploreown.shelf
summary: Items put on a shelf.
types:
  - {name: exploreown.shelf.ItemId, kind: newtype, of: Uuid}
entities:
  - name: exploreown.shelf.Item
    identity: {name: item_id, type: exploreown.shelf.ItemId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed], transitions: []}
actors:
  - name: exploreown.shelf.Keeper
    may: [exploreown.shelf.PutItem]
commands:
  - name: exploreown.shelf.PutItem
    input:
      - {name: label, type: String}
    outcomes:
      - name: put
        creates: exploreown.shelf.Item
        instance: item_id
        sets: {label: input.label}
        emits: [exploreown.shelf.ItemPut]
        payload:
          exploreown.shelf.ItemPut: {item_id: {generated: true}}
events:
  - name: exploreown.shelf.ItemPut
    fields: [{name: item_id, type: exploreown.shelf.ItemId}]
views:
  - name: exploreown.shelf.Items
    source: exploreown.shelf.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: exploreown.shelf.ItemId}
      - {name: label, type: String}
";

const MEMO: &str = r"domain: exploreown.memo
summary: Memos written down.
types:
  - {name: exploreown.memo.MemoId, kind: newtype, of: Uuid}
entities:
  - name: exploreown.memo.Memo
    identity: {name: memo_id, type: exploreown.memo.MemoId}
    fields:
      - {name: body, type: String}
    lifecycle: {initial: Written, states: [Written], terminal: [Written], transitions: []}
actors:
  - name: exploreown.memo.Scribe
    may: [exploreown.memo.WriteMemo]
commands:
  - name: exploreown.memo.WriteMemo
    input:
      - {name: body, type: String}
    outcomes:
      - name: written
        creates: exploreown.memo.Memo
        instance: memo_id
        sets: {body: input.body}
        emits: [exploreown.memo.MemoWritten]
        payload:
          exploreown.memo.MemoWritten: {memo_id: {generated: true}}
events:
  - name: exploreown.memo.MemoWritten
    fields: [{name: memo_id, type: exploreown.memo.MemoId}]
views:
  - name: exploreown.memo.Memos
    source: exploreown.memo.Memo
    consistency: read_your_writes
    fields:
      - {name: memo_id, type: exploreown.memo.MemoId}
      - {name: body, type: String}
";

/// The shelf component alone: `PutItem` and the shelf view. Mode `wrong-view` answers the shelf
/// view with no rows, which only an explorer that reads that view can notice.
const SHELF_TARGET: &str = r#"package essconform

import "fmt"

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtTarget struct {
	mode    string
	order   []string
	labels  map[string]string
	version int
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	t := &exploreExtTarget{mode: mode}
	t.reset()
	return t
}

func (t *exploreExtTarget) reset() {
	t.order = nil
	t.labels = map[string]string{}
	t.version = 0
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "adversary-shelf-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "exploreown.shelf.PutItem" {
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of the shelf component"}
	}
	label, _ := request.Input["label"].(string)
	id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.order)+1)
	t.order = append(t.order, id)
	t.labels[id] = label
	t.version++
	return CommandResult{
		Outcome:      "put",
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: "exploreown.shelf.ItemPut", Payload: map[string]Node{"item_id": id}}},
	}, nil
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "exploreown.shelf.Items" {
		return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of the shelf component"}
	}
	rows := []Row{}
	if t.mode == "wrong-view" {
		return ViewResult{Rows: rows}, nil
	}
	for _, id := range t.order {
		rows = append(rows, Row{"item_id": id, "label": t.labels[id]})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreExtUnsupported{"no external branches"}
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
"#;

fn compiled(files: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut raws = Vec::new();
    for (name, text) in files {
        sources.insert(*name, *text);
        raws.push((
            Source::new(*name),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name} parses: {error}")),
        ));
    }
    let specification = Specification::assemble(raws)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
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

/// A mutation of the emitted `explore.go`, named by `ESS_ADVERSARY_MUTANT`, applied to the copy in
/// scratch: `owns-commands` drops the owned-domain clause of the command filter, `owns-views` drops
/// every view.
fn mutated(explore: &str) -> String {
    let (from, to) = match std::env::var("ESS_ADVERSARY_MUTANT").as_deref() {
        Ok("owns-commands") => (
            r#"accepts[exploreString(command["name"])] || owns[exploreString(command["domain"])]"#,
            r#"accepts[exploreString(command["name"])]"#,
        ),
        Ok("owns-views") => ("views[key] = node", "_, _ = key, node"),
        _ => return explore.to_owned(),
    };
    assert!(
        explore.contains(from),
        "the mutant's site `{from}` is in explore.go"
    );
    explore.replacen(from, to, 1)
}

/// Runs the Go explorer over `suite` and `ir` against `target`, once per case: each case's result
/// and assertion line.
fn go_lane(
    label: &str,
    ir: &EssIr,
    suite: &ConformanceSuite,
    target: &str,
    cases: &Value,
) -> (
    BTreeMap<String, Option<Value>>,
    BTreeMap<String, String>,
    String,
) {
    let root = support_scratch::Scratch::new(format!(
        "ess-adversary-w2-exp1-{label}-{}",
        std::process::id()
    ));
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(suite, ir).unwrap() {
        let contents = if artifact.path.ends_with("/explore.go") {
            mutated(&artifact.contents)
        } else {
            artifact.contents
        };
        write(&module.join(&artifact.path), &contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/adversaryexp1\n\ngo 1.24\n",
    );
    write(
        &module.join("essconform/explore_external_target_test.go"),
        target,
    );
    std::fs::copy(
        fixture("explore_external_driver_test.go"),
        module.join("essconform/explore_external_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out");
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
    let log = format!(
        "{}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(run.status.success(), "{log}");
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json")))
            .ok()
            .map(|text| serde_json::from_str(&text).expect("JSON"));
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        results.insert(name.clone(), json);
        asserts.insert(name, assert.trim_end().to_owned());
    }
    (results, asserts, log)
}

/// A component that owns a domain and lists no command explores that domain's command and view,
/// and nothing of the other component's (story:component-explorer-walks-only-its-component).
#[test]
fn a_component_owning_a_domain_explores_its_commands_and_views_and_no_others() {
    let ir = compiled(&[
        ("system.yaml", SYSTEM),
        ("shelf.yaml", SHELF),
        ("memo.yaml", MEMO),
    ]);
    let mut suite = ess_conformance::synthesize::synthesize_for(&ir, "shelf-service")
        .unwrap_or_else(|unknown| panic!("{unknown}"))
        .suite;
    suite.select_fresh_format_for(&ir);
    let cases = json!([
        {"name": "shelf"},
        {"name": "wrong-view", "mode": "wrong-view"},
    ]);
    let (results, asserts, log) = go_lane("owns", &ir, &suite, SHELF_TARGET, &cases);
    let shelf = results["shelf"]
        .as_ref()
        .unwrap_or_else(|| panic!("`shelf`: {}\n{log}", asserts["shelf"]));
    assert_eq!(asserts["shelf"], "ok", "{shelf}");
    assert_eq!(shelf["excluded"], json!([]), "{shelf}");
    assert_eq!(
        shelf["reached"],
        json!(["exploreown.shelf.PutItem/put"]),
        "the owned domain's command is explored: {shelf}"
    );
    assert!(
        !shelf.to_string().contains("exploreown.memo."),
        "nothing of memo-service is explored: {shelf}"
    );
    let wrong = results["wrong-view"]
        .as_ref()
        .unwrap_or_else(|| panic!("`wrong-view`: {}\n{log}", asserts["wrong-view"]));
    let message = wrong["failure"]["message"].as_str().unwrap_or_default();
    assert!(
        asserts["wrong-view"].starts_with("failed: explore: seed ")
            && message.starts_with("view-rows: `exploreown.shelf.Items` holds 0 row(s)"),
        "the owned domain's view is read: {} {wrong}",
        asserts["wrong-view"]
    );
}
