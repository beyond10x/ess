//! A React chart names each series in a legend, by field label, and keeps its x labels inside
//! the chart's box (beyond10x/ess#357).

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOC: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: totals
  sections: [{name: all, pages: [totals]}]
pages:
  totals:
    kind: detail_page
    title: Totals
    sections:
      - name: by_stage
        component: chart
        chart: bar
        reads: {view: deals.StageTotals}
        x: stage
        series: [won_value, lost_value]
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-chart")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let doc = ess_ui::load_str(DOC).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["runtime/composites/chart.tsx", "runtime/data.ts"]);
    out
}

/// Renders a `kind` chart over three stages, one with a long name, and checks the legend and
/// the x labels.
const SCRIPT: &str = r#"
const { ChartView } = out("runtime/composites/chart");
const data = out("runtime/data");
const rows = [
  { stage: "lead", won_value: 1, lost_value: 2 },
  { stage: "a stage name far longer than any bar is wide", won_value: 3, lost_value: 1 },
  { stage: "won", won_value: 5, lost_value: 0 },
];
data.setDataAdapter({ ...data.fixtureAdapter, read: async () => ({ rows }) });
const root = React.__root(jsx(ChartView, {
  reads: { view: "deals.StageTotals" },
  chart: KIND,
  x: "stage",
  series: ["won_value", "lost_value"],
}));
(async () => {
  root.settle();
  await new Promise((resolve) => setTimeout(resolve, 5));
  root.settle();
  const has = (name) => (n) => n.props && typeof n.props.className === "string" && n.props.className.split(" ").includes(name);
  const [legend] = root.find(has("ui-chart-legend"));
  assert.ok(legend, "the chart has a legend");
  const items = root.find(has("ui-legend-item"));
  assert.strictEqual(items.length, 2, "one legend entry per series");
  const text = (n) => (n.children || []).map((c) => (typeof c === "string" ? c : text(c))).join("");
  assert.deepStrictEqual(items.map(text), ["won value", "lost value"], "each named by its field label");
  for (const [at, item] of items.entries()) {
    assert.ok(item.children.some((c) => typeof c === "object" && has(`ui-series-${at}`)(c)), `entry ${at} carries its series swatch`);
  }
  const labels = root.find(has("ui-chart-x-label"));
  assert.deepStrictEqual(labels.map(text), rows.map((r) => r.stage), "one x label per row");
  for (const label of labels) {
    assert.strictEqual(label.props.title, text(label), "a truncated label shows in full on hover");
  }
})().catch((error) => { console.error(error); process.exit(1); });
"#;

#[test]
fn a_bar_chart_has_a_series_legend_and_bounded_x_labels() {
    let out = project("bar");
    node(&out, "bar", &SCRIPT.replace("KIND", "\"bar\""));
    let css = std::fs::read_to_string(out.join("src/runtime/styles.css")).expect("the styles");
    let rule = css
        .lines()
        .find(|line| line.starts_with(".ui-chart-x-label"))
        .unwrap_or_else(|| panic!("no x label rule:\n{css}"));
    for property in [
        "overflow: hidden",
        "text-overflow: ellipsis",
        "white-space: nowrap",
    ] {
        assert!(rule.contains(property), "{rule}");
    }
}

#[test]
fn a_line_chart_has_a_series_legend_and_x_labels() {
    let out = project("line");
    node(&out, "line", &SCRIPT.replace("KIND", "\"line\""));
}

/// A chart's `x` and each `series` take a field as a column does (beyond10x/ess#723): with
/// `x: {field: at, label: Day, as: date}` each tick is a date and the axis is titled `Day`, and a
/// series `{field: count, label: Orders}` is titled `Orders` in the legend and the table header.
#[test]
fn a_chart_names_its_axes_and_formats_its_ticks() {
    let text = DOC
        .replace(
            "reads: {view: deals.StageTotals}",
            "reads: {view: deals.Events}",
        )
        .replace("x: stage", "x: {field: at, label: Day, as: date}")
        .replace(
            "series: [won_value, lost_value]",
            "series: [{field: count, label: Orders}]",
        );
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-chart")
        .join("axes");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let doc = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(out.join("src/pages/Totals.tsx")).expect("the page");
    for written in [
        r#"x={{ field: "at", label: "Day", as: "date" }}"#,
        r#"series={[{ field: "count", label: "Orders" }]}"#,
    ] {
        assert!(page.contains(written), "{written}: {page}");
    }
    compile(&out, &["runtime/composites/chart.tsx", "runtime/data.ts"]);
    node(
        &out,
        "axes",
        r#"
const { ChartView } = out("runtime/composites/chart");
const { formatValue } = out("runtime/core");
const data = out("runtime/data");
const rows = [{ at: "2026-01-02T00:00:00Z", count: 3 }, { at: "2026-01-03T00:00:00Z", count: 5 }];
data.setDataAdapter({ ...data.fixtureAdapter, read: async () => ({ rows }) });
const has = (name) => (n) => n.props && typeof n.props.className === "string" && n.props.className.split(" ").includes(name);
const text = (n) => (n.children || []).map((c) => (typeof c === "string" ? c : text(c))).join("");
const days = rows.map((row) => formatValue(row.at, "date"));
(async () => {
  for (const kind of ["bar", "line", "table"]) {
    const root = React.__root(jsx(ChartView, {
      reads: { view: "deals.Events" },
      chart: kind,
      x: { field: "at", label: "Day", as: "date" },
      series: [{ field: "count", label: "Orders" }],
    }));
    root.settle();
    await new Promise((resolve) => setTimeout(resolve, 5));
    root.settle();
    assert.notStrictEqual(days[0], rows[0].at, "a date is not its timestamp");
    assert.deepStrictEqual(root.find(has("ui-legend-item")).map(text), ["Orders"], kind);
    if (kind === "table") {
      assert.deepStrictEqual(root.find((n) => n.tag === "th").map(text), ["Day", "Orders"]);
      assert.deepStrictEqual(root.find((n) => n.tag === "tr").slice(1).map((r) => text(r.children[0])), days);
    } else {
      assert.deepStrictEqual(root.find(has("ui-chart-x-label")).map(text), days, kind);
      assert.deepStrictEqual(root.find(has("ui-chart-x-title")).map(text), ["Day"], kind);
    }
  }
})().catch((error) => { console.error(error); process.exit(1); });
"#,
    );
}

/// A chart draws its rows in the order its `sort` declares, after the live effects (beyond10x/ess#723):
/// three read rows and two `insert_or_patch` events, which put each new row first, draw 1 to 5 under
/// `sort: {by: seq}` in a line, a bar and a table chart, 5 to 1 under `dir: desc`, equal keys in
/// their order, and 5, 4, 1, 2, 3 (rows as the effects leave them) without a sort.
#[test]
fn a_chart_draws_its_rows_in_the_order_its_sort_declares() {
    let text = DOC
        .replace("pages:\n  totals:", "channels: {points: {carries: {events: [probe.Point]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  totals:")
        .replace("x: stage", "x: seq\n        sort: {by: seq}\n        live: {channel: points, effect: insert_or_patch}")
        .replace("series: [won_value, lost_value]", "series: [value]");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-chart")
        .join("sort");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let doc = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(out.join("src/pages/Totals.tsx")).expect("the page");
    assert!(page.contains(r#"sort={{ by: "seq" }}"#), "{page}");
    compile(
        &out,
        &[
            "runtime/composites/chart.tsx",
            "runtime/data.ts",
            "runtime/live.ts",
            "runtime/core.tsx",
        ],
    );
    node(
        &out,
        "sort",
        r#"
const { ChartView } = out("runtime/composites/chart");
const { DataScope } = out("runtime/core");
const { useLive, useNetworkChannels } = out("runtime/live");
let source;
global.EventSource = class { constructor() { source = this; } close() {} };
useNetworkChannels("http://app.test");
const read = [1, 2, 3].map((seq) => ({ id: String(seq), seq, parity: seq % 2, value: seq }));
const has = (name) => (n) => n.props && typeof n.props.className === "string" && n.props.className.split(" ").includes(name);
const text = (n) => (n.children || []).map((c) => (typeof c === "string" ? c : text(c))).join("");
const drawn = (root, kind) => kind === "table"
  ? root.find((n) => n.tag === "tr").slice(1).map((r) => text(r.children[0]))
  : root.find(has("ui-chart-x-label")).map(text);
const cases = [
  [{ by: "seq" }, ["1", "2", "3", "4", "5"]],
  [{ by: "seq", dir: "desc" }, ["5", "4", "3", "2", "1"]],
  [{ by: "parity" }, ["4", "2", "5", "1", "3"]],
  [undefined, ["5", "4", "1", "2", "3"]],
];
for (const kind of ["line", "bar", "table"]) {
  for (const [sort, expected] of cases) {
    function Live() {
      const raw = useLive({ status: "ready", rows: read, params: {}, refetch() {} }, { channel: "points", effect: "insert_or_patch" });
      return jsx(DataScope, { data: raw, children: jsx(ChartView, { chart: kind, x: "seq", series: ["value"], sort }) });
    }
    const root = React.__root(jsx(Live, {})).settle();
    for (const seq of [4, 5]) {
      source.onmessage({ data: JSON.stringify({ event: "probe.Point", payload: { id: String(seq), seq, parity: seq % 2, value: seq } }) });
      root.settle();
    }
    assert.deepStrictEqual(drawn(root, kind), expected, `${kind} ${JSON.stringify(sort)}`);
  }
}
"#,
    );
}
