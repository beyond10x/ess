//! A graph editor whose nodes and edges live in two views: `reads` holds the nodes, keyed and
//! labelled by `nodes.key` and `nodes.label`, and `edges.reads` the edges (beyond10x/ess#352).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: plan
model: plan.system
placement_profile: fat
fixtures: {views: {tasks.All: data.yaml, dependencies.All: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: tasks.graph
  sections: [{name: all, pages: [tasks.graph]}]
pages:
  tasks.graph:
    kind: detail_page
    title: Task graph
    sections:
      - name: graph
        component: graph_editor
        reads: {view: tasks.All}
        nodes: {key: task_id, label: title}
        edges: {reads: {view: dependencies.All}, from: from_task_id, to: to_task_id}
";

const DATA: &str = r"
views:
  tasks.All:
    rows:
      - {task_id: t-1, title: Design the schema}
      - {task_id: t-2, title: Build the importer}
      - {task_id: t-3, title: Ship the release}
  dependencies.All:
    rows:
      - {id: d-1, from_task_id: t-1, to_task_id: t-2}
      - {id: d-2, from_task_id: t-2, to_task_id: t-3}
";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-graph")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

#[test]
fn nodes_and_edges_from_two_views_draw_one_graph() {
    let base = dir("two-views");
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("tasks.graph", &[]);
    let screen = app.render_text(120, 40);
    for title in [
        "Design the schema",
        "Build the importer",
        "Ship the release",
    ] {
        assert!(
            screen.contains(title),
            "a node is labelled by nodes.label:\n{screen}"
        );
    }
    assert!(
        screen.contains("Design the schema → Build the importer"),
        "an edge joins the labels of the nodes its endpoints key:\n{screen}"
    );
    assert!(
        screen.contains("Build the importer → Ship the release"),
        "{screen}"
    );
    assert_eq!(
        app.rows("graph").len(),
        3,
        "the section's rows are the nodes"
    );
}
