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
