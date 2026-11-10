//! A graph editor draws its nodes in layers and each edge as a line between its two nodes, and a
//! node's or an edge's actions are a menu opened from one control (beyond10x/ess#723).
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

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
        edges: {from: from_step, to: to_step, kind_by: kind}
        node_actions:
          - {name: remove, does: steps.Remove, bind: {step: row.id}, label: Remove}
        edge_actions:
          - {name: unlink, does: steps.Unlink, bind: {edge: row.id}, label: Unlink}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-graph-editor")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let doc = ess_ui::load_str(DOC).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    compile(
        &out,
        &["runtime/composites/graph_editor.tsx", "runtime/data.ts"],
    );
    out
}

const HARNESS: &str = r#"
const { GraphEditorView } = out("runtime/composites/graph_editor");
const data = out("runtime/data");
const rows = [
  { id: "s1", label: "Start" },
  { id: "s2", label: "Middle" },
  { id: "s3", label: "End" },
  { id: "s4", label: "Aside" },
  { id: "e1", from_step: "s1", to_step: "s2", kind: "next" },
  { id: "e2", from_step: "s2", to_step: "s3", kind: "next" },
  { id: "e3", from_step: "s3", to_step: "s1", kind: "retry" },
];
const sent = [];
data.setDataAdapter({
  ...data.fixtureAdapter,
  read: async () => ({ rows }),
  send: async (command, input) => { sent.push([command, input]); return { accepted: true }; },
});
const base = "pages/steps.map/sections/editor";
const props = {
  "data-ui-path": base,
  reads: { view: "steps.All" },
  nodes: { label: "title" },
  edges: { from: "from_step", to: "to_step", kindBy: "kind" },
  nodeActions: [{ "data-ui-path": `${base}/node_actions/remove`, name: "remove", label: "Remove", does: "steps.Remove", bind: { step: "row.id" } }],
  edgeActions: [{ "data-ui-path": `${base}/edge_actions/unlink`, name: "unlink", label: "Unlink", does: "steps.Unlink", bind: { edge: "row.id" } }],
};
const root = React.__root(jsx(GraphEditorView, props));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const has = (name) => (n) => n.props && n.props.className && String(n.props.className).split(" ").includes(name);
const text = (n) => (typeof n === "string" ? n : (n.children || []).map(text).join(""));
const tag = (name) => (n) => n.tag === name;
const within = (n, test) => (typeof n === "string" ? [] : [n, ...(n.children || []).flatMap((c) => within(c, test))]).filter((m) => typeof m !== "string" && test(m));
"#;

fn run(name: &str, body: &str) {
    let out = project(name);
    node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exit(1); }});\n"
        ),
    );
}

#[test]
fn nodes_and_edges_are_drawn_as_a_graph() {
    run(
        "drawing",
        r#"
  await settle();
  const nodes = root.find(has("ui-graph-node"));
  const place = (n) => ({ left: n.props.style.left, top: n.props.style.top });
  const at = Object.fromEntries(nodes.map((n) => [n.props["data-ui-path"].split("/").pop(), place(n)]));
  assert.deepStrictEqual(Object.keys(at), ["s1", "s2", "s3", "s4"], "nodes keep row order");
  assert.ok(at.s1.left < at.s2.left && at.s2.left < at.s3.left, `layers run along the longest path: ${JSON.stringify(at)}`);
  assert.strictEqual(at.s4.left, at.s1.left, "a node no edge enters is in the first layer");
  assert.ok(at.s1.top < at.s4.top, "ties in a layer keep row order");

  const lines = root.find(tag("line"));
  assert.strictEqual(lines.length, 3, "one line per edge, the cycle-closing one too");
  assert.strictEqual(root.find(tag("svg")).length, 1, "one drawing");
  const labels = root.find(tag("text")).map(text);
  assert.deepStrictEqual(labels, ["next", "next", "retry"], "edges are labelled by kind_by");
  const first = lines[0].props;
  const s1 = nodes[0].props.style;
  const s2 = nodes[1].props.style;
  assert.ok(Math.abs(first.x1 - (s1.left + s1.width)) <= 1 && Math.abs(first.x2 - s2.left) <= 1, "an edge runs from one node to the other");

  const edges = root.find(has("ui-graph-edge"));
  assert.strictEqual(edges.length, 3, "the edge list stays as the drawing's text");
  assert.ok(text(edges[0]).startsWith("Start → Middle"), text(edges[0]));

  const shape = (r) => JSON.stringify(r.tree, (k, v) => (typeof v === "function" ? undefined : v));
  const before = shape(root);
  const again = React.__root(jsx(GraphEditorView, props));
  for (let i = 0; i < 4; i++) { again.settle(); await tick(); }
  again.settle();
  assert.strictEqual(shape(again), before, "the same rows give the same drawing");
"#,
    );
}

#[test]
fn node_actions_are_a_menu() {
    run(
        "menu",
        r#"
  await settle();
  const item = (row, kind, action) => root.find((n) => n.props && n.props["data-ui-path"] === `${base}/rows/${row}/${kind}/${action}`);
  assert.strictEqual(root.find(has("ui-graph-menu")).length, 4 + 3, "one menu control per node and per edge");
  assert.strictEqual(item("s2", "node_actions", "remove").length, 0, "a closed menu lists nothing");
  assert.strictEqual(root.find(has("ui-menu")).length, 0);

  root.find(has("ui-graph-menu"))[1].props.onClick();
  await settle();
  assert.strictEqual(root.find(has("ui-menu")).length, 1, "opening lists the menu");
  const remove = item("s2", "node_actions", "remove");
  assert.strictEqual(remove.length, 1, "the action keeps its path under the node's row");
  assert.strictEqual(text(remove[0]), "Remove");
  remove[0].props.onClick({ preventDefault() {} });
  await settle();
  assert.deepStrictEqual(sent, [["steps.Remove", { step: "s2" }]], "choosing runs it with the node as row");

  const edge = root.find(has("ui-graph-edge"))[1];
  within(edge, has("ui-graph-menu"))[0].props.onContextMenu({ preventDefault() {} });
  await settle();
  const unlink = item("e2", "edge_actions", "unlink");
  assert.strictEqual(unlink.length, 1, "an edge's actions are a menu too, opened by the context-menu key");
  unlink[0].props.onClick({ preventDefault() {} });
  await settle();
  assert.deepStrictEqual(sent[1], ["steps.Unlink", { edge: "e2" }], "choosing runs it with the edge as row");
"#,
    );
}

/// Adversary (beyond10x/ess#723): the cycle-closing edge is the one met last when walking from
/// the nodes no edge enters, not the one whose row comes last. With a source `S`, edges listed
/// `B -> A`, `A -> B`, `S -> A`, the longest path from the source is `S -> A -> B`, so `B` is in the
/// third layer; it is not a first-layer node, since an edge enters it.
#[test]
fn adversary_layers_run_from_the_nodes_no_edge_enters_whatever_the_edge_row_order() {
    run(
        "adversary-layers",
        r#"
  rows.length = 0;
  rows.push(
    { id: "S", label: "Source" },
    { id: "A", label: "Ask" },
    { id: "B", label: "Branch" },
    { id: "e1", from_step: "B", to_step: "A", kind: "back" },
    { id: "e2", from_step: "A", to_step: "B", kind: "next" },
    { id: "e3", from_step: "S", to_step: "A", kind: "next" },
  );
  await settle();
  const nodes = root.find(has("ui-graph-node"));
  const at = Object.fromEntries(nodes.map((n) => [n.props["data-ui-path"].split("/").pop(), n.props.style.left]));
  assert.ok(at.S < at.A && at.A < at.B, `layers run along the longest path from the source S: ${JSON.stringify(at)}`);
"#,
    );
}

/// Adversary (beyond10x/ess#723): an edge from a node to itself (a retry loop) is drawn where it
/// can be seen: its label does not sit inside its own node's box, which is painted over the
/// drawing.
#[test]
fn adversary_a_self_loop_is_drawn_outside_its_node() {
    run(
        "adversary-self-loop",
        r#"
  rows.length = 0;
  rows.push(
    { id: "s1", label: "Start" },
    { id: "s2", label: "Poll" },
    { id: "e1", from_step: "s1", to_step: "s2", kind: "next" },
    { id: "e2", from_step: "s2", to_step: "s2", kind: "retry" },
  );
  await settle();
  const node = root.find(has("ui-graph-node"))[1].props.style;
  const label = root.find(tag("text")).find((t) => text(t) === "retry");
  assert.ok(label, "the self-loop is labelled");
  const { x, y } = label.props;
  const inside = x >= node.left && x <= node.left + node.width && y >= node.top && y <= node.top + node.height;
  assert.ok(!inside, `the self-loop's label at ${x},${y} is hidden under its node ${JSON.stringify(node)}`);
"#,
    );
}
