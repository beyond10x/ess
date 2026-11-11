//! Adversary (beyond10x/ess#723): a graph editor whose edges come from a view of their own
//! (`edges.reads`) can hold a node and an edge with the same key, as two tables numbered from 1
//! do. The generated React page then has two rows at `<editor>/rows/1`, the node first. The
//! Playwright export opens "the menu of the row" before choosing an action, so the menu it opens
//! for `rows/1/edge_actions/unlink` has to be the edge's, not the node's that precedes it.

use std::path::{Path, PathBuf};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {steps.All: steps.yaml, links.All: links.yaml}}
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
        edges: {reads: {view: links.All}, from: from_step, to: to_step}
        node_actions:
          - {name: remove, does: steps.Remove, bind: {step: row.id}, label: Remove}
        edge_actions:
          - {name: unlink, does: links.Unlink, bind: {link: row.id}, label: Unlink}
";

const STEPS: &str = "views:\n  steps.All:\n    rows:\n      - {id: 1, title: Greeting}\n      - {id: 2, title: Menu}\n";
const LINKS: &str = "views:\n  links.All:\n    rows:\n      - {id: 1, from_step: 1, to_step: 2}\n";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adversary-graph-menu")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("steps.yaml"), STEPS).expect("the fixture is written");
    std::fs::write(dir.join("links.yaml"), LINKS).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn spec(name: &str, steps: &[&str]) -> String {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: graph menu\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("graph_menu.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    ess_ui_test::playwright(
        &[file],
        &ess_ui::load_path(&document(name)).expect("the document loads"),
    )
}

fn opener(spec: &str, after: &str) -> String {
    let lines: Vec<&str> = spec.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.contains(after))
        .unwrap_or_else(|| panic!("no line for {after}:\n{spec}"));
    lines[..at]
        .iter()
        .rev()
        .find(|line| line.contains(".ui-graph-menu"))
        .unwrap_or_else(|| panic!("no menu is opened before {after}:\n{spec}"))
        .trim()
        .to_string()
}

#[test]
fn adversary_an_edge_action_opens_the_edges_menu_when_a_node_shares_its_key() {
    let node = "pages/steps.map/sections/editor/rows/1/node_actions/remove";
    let edge = "pages/steps.map/sections/editor/rows/1/edge_actions/unlink";
    let spec = spec(
        "shared-key",
        &[
            "open: steps.map",
            &format!("act: {node}"),
            &format!("act: {edge}"),
        ],
    );
    let for_node = opener(&spec, &format!("[data-ui-path=\"{node}\"]"));
    let for_edge = opener(&spec, &format!("[data-ui-path=\"{edge}\"]"));
    assert_ne!(
        for_node, for_edge,
        "the edge action opens the same menu as the node action, the node's (first in the page):\n{spec}"
    );
}
