//! What an `ess-ui/1` document can say, honoured by the generated React project: literal text
//! through a widget argument (beyond10x/ess#353), a declared group order (#351), a metric over the
//! rows of its read (#358) and a column labelled from a related view (#364).

mod support;

use std::path::{Path, PathBuf};

const DOCUMENT: &str = "format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: home
  sections: [{name: all, pages: [home]}]
pages:
  home:
    kind: static_page
    title: Home
    sections:
      - name: pipeline
        component: collection
        reads: {view: objectives.All}
        columns:
          - goal
          - {field: owner_id, label_from: {view: people.All, field: name, key: person_id}}
        group_by: state
        group_order: [Draft, Done]
        show_empty_groups: true
      - {name: owed, component: metric, reads: {view: invoices.All}, aggregate: sum, field: amount, label: Owed}
";

fn generated(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-document-model")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    let document = dir.join("ui.yaml");
    std::fs::write(&document, DOCUMENT).expect("the document is written");
    let out = dir.join("out");
    ess_ui_react::generate_path(&document, &out).unwrap_or_else(|error| panic!("{error}"));
    out
}

fn sources(project: &Path) -> String {
    let mut text = String::new();
    let mut pending = vec![project.join("src")];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("the dir reads") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext == "tsx" || ext == "ts")
            {
                text.push_str(&std::fs::read_to_string(&path).expect("the file reads"));
            }
        }
    }
    text
}

/// The new keys reach the generated page as the props the runtime reads.
#[test]
fn the_page_carries_group_order_aggregates_and_label_lookups() {
    let project = generated("props");
    let text = sources(&project);
    for prop in [
        "groupOrder={[\"Draft\", \"Done\"]}",
        "showEmptyGroups={true}",
        "aggregate={\"sum\"}",
        "field={\"amount\"}",
        "labelFrom: { view: \"people.All\", field: \"name\", key: \"person_id\" }",
    ] {
        assert!(text.contains(prop), "`{prop}` is generated");
    }
}

/// #353, #351, #358, #364 in the runtime: literal text, group order, aggregates, label lookup.
#[test]
fn the_runtime_evaluates_text_groups_aggregates_and_labels() {
    let project = generated("runtime");
    support::compile(
        &project,
        &[
            "runtime/expr.ts",
            "runtime/composites/collection.tsx",
            "runtime/composites/metric.tsx",
            "runtime/fields.tsx",
        ],
    );
    support::node(
        &project,
        "runtime",
        r#"
const { evaluate } = out("runtime/expr");
assert.strictEqual(evaluate("Limit in cents", {}), "Limit in cents");
assert.strictEqual(evaluate("not yet sent", {}), "not yet sent");
assert.strictEqual(evaluate("Terms and conditions", {}), "Terms and conditions");
assert.strictEqual(evaluate("row.state in [Done, Draft]", { row: { state: "Done" } }), true);
assert.strictEqual(evaluate("not state.paused", { state: {} }), true);
assert.strictEqual(evaluate("false", {}), false);

const { groupRows } = out("runtime/composites/collection");
const rows = [
  { id: "o1", state: "Done" },
  { id: "o2", state: "Parked" },
  { id: "o3", state: "Draft" },
];
assert.deepStrictEqual(
  groupRows(rows, "state", ["Draft", "Verifying", "Done"], false).map(([g, r]) => [g, r.length]),
  [["Draft", 1], ["Done", 1], ["Parked", 1]],
);
assert.deepStrictEqual(
  groupRows(rows, "state", ["Draft", "Verifying", "Done"], true).map(([g, r]) => [g, r.length]),
  [["Draft", 1], ["Verifying", 0], ["Done", 1], ["Parked", 1]],
);
assert.deepStrictEqual(groupRows(rows, "state").map(([g]) => g), ["Done", "Parked", "Draft"]);

const { aggregateRows } = out("runtime/composites/metric");
const amounts = [{ amount: 10 }, { amount: 30 }, { amount: 5 }, { amount: 15 }];
assert.strictEqual(aggregateRows(amounts, "count"), 4);
assert.strictEqual(aggregateRows(amounts, "sum", "amount"), 60);
assert.strictEqual(aggregateRows(amounts, "min", "amount"), 5);
assert.strictEqual(aggregateRows(amounts, "max", "amount"), 30);
assert.strictEqual(aggregateRows(amounts, "avg", "amount"), 15);
assert.strictEqual(aggregateRows([], "avg", "amount"), undefined);

const { lookupLabel } = out("runtime/fields");
const people = [{ person_id: "p-9", name: "Ada" }];
assert.strictEqual(lookupLabel(people, { view: "people.All", field: "name", key: "person_id" }, "p-9"), "Ada");
assert.strictEqual(lookupLabel(people, { view: "people.All", field: "name", key: "person_id" }, "p-1"), "p-1");
assert.strictEqual(lookupLabel([{ id: "x", goal: "Ship" }], { view: "o", field: "goal" }, "x"), "Ship");
"#,
    );
}
