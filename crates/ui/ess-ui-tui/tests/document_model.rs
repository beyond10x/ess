//! What an `ess-ui/1` document can say, honoured by the terminal: literal text through a widget
//! argument (beyond10x/ess#353), a declared group order (#351), a metric over the rows of its
//! read (#358) and a column labelled from a related view (#364).

use std::collections::BTreeMap;
use std::path::Path;

use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use serde_yaml::Value;

/// Answers every read from rows held in memory, by view.
struct Rows(BTreeMap<String, Vec<Value>>);

impl DataAdapter for Rows {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        Ok(ReadResult {
            rows: self.0.get(&request.view).cloned().unwrap_or_default(),
            total: None,
        })
    }
    fn run(&mut self, _: &str, _: &BTreeMap<String, Value>) -> Result<String, String> {
        Ok(String::new())
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

fn state_dir(test: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-document-model")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

/// A one-page document whose page holds `sections` (YAML, indented under `sections:`), with
/// `widgets` declared at the top level.
fn document(widgets: &str, sections: &str) -> String {
    format!(
        "format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      nav:     {{kind: navigation}}
      main:    {{kind: page_outlet}}
      overlay: {{kind: overlay_outlet}}
navigation:
  home: home
  sections: [{{name: all, pages: [home]}}]
widgets:
{widgets}
pages:
  home:
    kind: static_page
    title: Home
    sections:
{sections}"
    )
}

fn rows(yaml: &str) -> Vec<Value> {
    serde_yaml::from_str(yaml).expect("the rows parse")
}

fn screen(document: &str, data: &[(&str, &str)], test: &str) -> String {
    let document = ess_ui::load_str(document).unwrap_or_else(|error| panic!("{error}"));
    let views = data
        .iter()
        .map(|(view, yaml)| ((*view).to_owned(), rows(yaml)))
        .collect();
    let app = App::with_adapter(
        document,
        Box::new(Rows(views)),
        Vec::new(),
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.render_text(140, 50)
}

const NO_WIDGETS: &str = "  {}";

/// #353: a widget argument that is a sentence, not an expression, is shown as written. Words
/// joined by `not`, `in` or `==` read no path, so they are text, as `ess ui check` already says.
#[test]
fn a_literal_widget_argument_is_shown_as_text() {
    let widgets = "  caption:
    summary: A caption.
    params: {label: {type: string, required: true, note: the text}}
    body:
      - {name: text, primitive: text, text: args.label}";
    let sections = "      - name: notes
        component: record
        reads: {view: notes.One}
        fields: [id]
        item:
          - {name: first, component: caption, args: {label: not yet sent}}
          - {name: second, component: caption, args: {label: Limit in cents}}
          - {name: third, component: caption, args: {label: row.id}}";
    let text = screen(
        &document(widgets, sections),
        &[("notes.One", "[{id: n-1}]")],
        "literal-arg",
    );
    assert!(text.contains("not yet sent"), "{text}");
    assert!(text.contains("Limit in cents"), "{text}");
    assert!(!text.contains("false"), "{text}");
}

/// #351: groups follow `group_order`, not the order the rows arrive in, and with
/// `show_empty_groups` a declared group without rows keeps its heading.
#[test]
fn groups_follow_the_declared_order_and_empty_ones_can_show() {
    let sections = "      - name: pipeline
        component: collection
        reads: {view: objectives.All}
        columns: [goal]
        group_by: state
        group_order: [Draft, Verifying, Done, Escalated]
        show_empty_groups: true";
    let text = screen(
        &document(NO_WIDGETS, sections),
        &[(
            "objectives.All",
            "[{id: o1, goal: Ship it, state: Done}, {id: o2, goal: Plan it, state: Draft},
              {id: o3, goal: Check it, state: Verifying}, {id: o4, goal: Odd one, state: Parked}]",
        )],
        "group-order",
    );
    let at = |needle: &str| {
        text.find(needle)
            .unwrap_or_else(|| panic!("`{needle}` is shown:\n{text}"))
    };
    let order = [
        at("state: Draft"),
        at("state: Verifying"),
        at("state: Done"),
        at("state: Escalated"),
        at("state: Parked"),
    ];
    assert!(
        order.windows(2).all(|pair| pair[0] < pair[1]),
        "declared order, then the rest:\n{text}"
    );
}

/// #358: a metric counts, sums, or takes the min, max or average of a field over its read's rows.
#[test]
fn a_metric_aggregates_the_rows_of_its_read() {
    let sections = "      - {name: open, component: metric, reads: {view: invoices.All}, aggregate: count, label: Open invoices}
      - {name: owed, component: metric, reads: {view: invoices.All}, aggregate: sum, field: amount, label: Owed}
      - {name: least, component: metric, reads: {view: invoices.All}, aggregate: min, field: amount, label: Least}
      - {name: most, component: metric, reads: {view: invoices.All}, aggregate: max, field: amount, label: Most}
      - {name: mean, component: metric, reads: {view: invoices.All}, aggregate: avg, field: amount, label: Mean}";
    let text = screen(
        &document(NO_WIDGETS, sections),
        &[(
            "invoices.All",
            "[{id: i1, amount: 10}, {id: i2, amount: 30}, {id: i3, amount: 5}, {id: i4, amount: 15}]",
        )],
        "metric-aggregate",
    );
    for shown in [
        "Open invoices: 4",
        "Owed: 60",
        "Least: 5",
        "Most: 30",
        "Mean: 15",
    ] {
        assert!(text.contains(shown), "`{shown}`:\n{text}");
    }
}

/// #364: a column shows a field of a related view, matched by key, instead of the raw key.
#[test]
fn a_column_is_labelled_from_a_related_view() {
    let sections = "      - name: tasks
        component: collection
        reads: {view: tasks.All}
        columns:
          - title
          - {field: objective_id, label: Objective, label_from: {view: objectives.All, field: goal}}
          - {field: owner_id, label_from: {view: people.All, field: name, key: person_id}}";
    let text = screen(
        &document(NO_WIDGETS, sections),
        &[
            (
                "tasks.All",
                "[{id: t1, title: Write, objective_id: 7f3a, owner_id: p-9},
                  {id: t2, title: Review, objective_id: 99zz, owner_id: p-1}]",
            ),
            (
                "objectives.All",
                "[{id: 7f3a, goal: Ship the console}, {id: 1b2c, goal: Other}]",
            ),
            ("people.All", "[{person_id: p-9, name: Ada}]"),
        ],
        "label-from",
    );
    assert!(text.contains("Ship the console"), "{text}");
    assert!(
        !text.contains("7f3a"),
        "the key is replaced by its label:\n{text}"
    );
    assert!(text.contains("Ada"), "{text}");
    assert!(
        text.contains("99zz"),
        "a key the related view lacks is shown as written:\n{text}"
    );
}
