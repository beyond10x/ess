//! A graph editor's node action with an inline confirm is addressed under the node that opened
//! it, `<editor>/rows/<key>/node_actions/<action>/confirm/overlay`, in the terminal and in the
//! Playwright spec, as a collection's row action is. The generated React project scopes every
//! node's actions under `rows/<key>`, so a path without the row matched nothing in the browser
//! while it was the only one the terminal accepted.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {steps.All: steps.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: steps.map
  sections: [{name: all, pages: [steps.map]}]
pages:
  steps.map:
    kind: detail_page
    title: Step map
    sections:
      - name: editor
        component: graph_editor
        reads: {view: steps.All}
        nodes: {label: title}
        node_actions:
          - {name: remove, does: steps.Remove, bind: {step: row.id}, label: Remove, confirm: {title: Remove step?}}
";

const DATA: &str = "views:\n  steps.All:\n    rows:\n      - {id: s-1, title: Greeting}\n      - {id: s-2, title: Menu}\n";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-graph-confirm")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("steps.yaml"), DATA).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn file(name: &str, steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: graph confirm\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("graph_confirm.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let report = ess_ui_test::execute(&document(name), &[file(name, steps)])
        .unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(name: &str, steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(name, steps)],
        &ess_ui::load_path(&document(name)).expect("the document loads"),
    )
}

const ACTION: &str = "pages/steps.map/sections/editor/rows/s-2/node_actions/remove";
const CONFIRM: &str =
    "pages/steps.map/sections/editor/rows/s-2/node_actions/remove/confirm/overlay";

#[test]
fn a_node_actions_confirm_is_addressed_under_its_node_and_confirming_runs_the_action() {
    let steps = [
        "open: steps.map",
        &format!("act: {ACTION}"),
        &format!("expect: {{at: {CONFIRM}, text: Remove step?}}"),
        &format!("act: {CONFIRM}"),
        "expect_command: {command: steps.Remove, input: {step: s-2}}",
    ];
    let outcome = run("under-node", &steps);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = spec("under-node", &steps);
    let at = format!("[data-ui-path=\"{CONFIRM}\"]");
    assert!(
        spec.contains(&format!(
            "await expect(page.locator('{at}')).toContainText(\"Remove step?\");"
        )),
        "{spec}"
    );
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn another_nodes_confirm_is_not_open() {
    let outcome = run(
        "another-node",
        &[
            "open: steps.map",
            &format!("act: {ACTION}"),
            "expect: {at: pages/steps.map/sections/editor/rows/s-1/node_actions/remove/confirm/overlay, text: Remove step?}",
        ],
    );
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    assert!(
        outcome.message.unwrap_or_default().contains("is not open"),
        "a confirm opened from s-2 is not s-1's"
    );
}

#[test]
fn a_node_actions_confirm_without_its_node_is_refused_in_both_renderers() {
    let steps = [
        "open: steps.map",
        &format!("act: {ACTION}"),
        "act: pages/steps.map/sections/editor/node_actions/remove/confirm/overlay",
    ];
    let outcome = run("without-node", &steps);
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("under its row"), "{message}");
    assert!(spec("without-node", &steps).contains("under its row"));
}
