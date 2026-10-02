//! A graph editor whose nodes and edges live in two views: `reads` holds the nodes, keyed and
//! labelled by `nodes.key` and `nodes.label`, and `edges.reads` the edges (beyond10x/ess#352).

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOC: &str = r"
format: ess-ui/1
app: plan
model: plan.system
placement_profile: fat
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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn nodes_and_edges_from_two_views_draw_one_graph() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-graph")
        .join("two-views");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let doc = ess_ui::load_str(DOC).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(out.join("src/pages/TasksGraph.tsx")).expect("the page");
    assert!(
        page.contains(r#"view: "dependencies.All""#),
        "the page passes the edge read:\n{page}"
    );
    compile(
        &out,
        &["runtime/composites/graph_editor.tsx", "runtime/data.ts"],
    );
    node(
        &out,
        "two-views",
        r#"
const { GraphEditorView } = out("runtime/composites/graph_editor");
const data = out("runtime/data");
const views = {
  "tasks.All": [
    { task_id: "t-1", title: "Design the schema" },
    { task_id: "t-2", title: "Build the importer" },
  ],
  "dependencies.All": [{ id: "d-1", from_task_id: "t-1", to_task_id: "t-2" }],
};
data.setDataAdapter({ ...data.fixtureAdapter, read: async (request) => ({ rows: views[request.view] || [] }) });
const root = React.__root(jsx(GraphEditorView, {
  "data-ui-path": "pages/tasks.graph/sections/graph",
  reads: { view: "tasks.All" },
  nodes: { key: "task_id", label: "title" },
  edges: { reads: { view: "dependencies.All" }, from: "from_task_id", to: "to_task_id" },
}));
(async () => {
  root.settle();
  await new Promise((resolve) => setTimeout(resolve, 5));
  root.settle();
  const has = (name) => (n) => n.props && n.props.className && String(n.props.className).split(" ").includes(name);
  const text = (n) => (n.children || []).map((c) => (typeof c === "string" ? c : text(c))).join("");
  const nodes = root.find(has("ui-graph-node"));
  assert.deepStrictEqual(nodes.map((n) => text(n.children[0])), ["Design the schema", "Build the importer"], "nodes labelled by nodes.label");
  assert.deepStrictEqual(nodes.map((n) => n.props["data-ui-path"]), [
    "pages/tasks.graph/sections/graph/rows/t-1",
    "pages/tasks.graph/sections/graph/rows/t-2",
  ], "nodes keyed by nodes.key");
  const edges = root.find(has("ui-graph-edge"));
  assert.strictEqual(edges.length, 1, "edges come from their own read");
  assert.ok(text(edges[0]).startsWith("Design the schema → Build the importer"), text(edges[0]));
})().catch((error) => { console.error(error); process.exit(1); });
"#,
    );
}
