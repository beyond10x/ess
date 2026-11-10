//! A metric over a view whose read failed draws an error state that names the status, a metric
//! whose read is loading draws a loading state, and a zero is shown only for an answered, empty
//! view.
//!
//! The generated page runs under `node` against the stand-in `react` of `support`, with a data
//! adapter that answers the view with a chosen status.

mod support;

use std::path::{Path, PathBuf};

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: shelf
  sections: [{name: all, pages: [shelf]}]
pages:
  shelf:
    kind: detail_page
    title: Shelf
    header:
      title: Shelf
      metrics:
        - name: items
          component: metric
          reads: {view: store.shelf.Items}
          aggregate: count
          label: Items on the shelf
        - name: units
          component: metric
          reads: {view: store.shelf.Items}
          aggregate: sum
          field: qty
          label: Units on the shelf
    sections:
      - name: list
        component: collection
        reads: {view: store.shelf.Items}
        columns: [qty]
";

const HARNESS: &str = r"
browser('/shelf');
const { setDataAdapter, DataError } = out('runtime/data');
const { ShelfPage } = out('pages/Shelf');
let answer = { status: 200, rows: [{ qty: 40 }, { qty: 55 }] };
setDataAdapter({ read: async () => {
  if (answer.status !== 200) throw new DataError(`${answer.status} store.shelf.Items`, answer.status);
  return { rows: answer.rows.map((row) => ({ ...row })), total: answer.rows.length };
}});
const root = React.__root(jsx(ShelfPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === 'string') return node;
  return (node.children || []).map(textOf).join('');
}
const metric = (label) => {
  const found = root.find((node) => node.tag === 'div' && node.props.className === 'ui-metric' && textOf(node).includes(label));
  assert.equal(found.length, 1, `one metric labelled ${label}: ${root.text()}`);
  return found[0];
};
const alerts = (node) => (node.children || []).filter((child) => child.props && child.props.role === 'alert');
";

fn run(test: &str, body: &str) {
    let out: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-metric-states")
        .join(test);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(DOCUMENT).expect("the document loads");
    ess_ui_react::generate(&document, Path::new("."), &out).expect("the project generates");
    support::compile(&out, &["pages/Shelf.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn a_metric_over_a_failed_view_draws_an_alert_naming_the_status_not_zero() {
    run(
        "failed",
        r"
  answer = { status: 501, rows: [] };
  await settle();
  for (const label of ['Items on the shelf', 'Units on the shelf']) {
    const node = metric(label);
    const found = alerts(node);
    assert.equal(found.length, 1, `${label} draws one alert: ${textOf(node)}`);
    assert.ok(textOf(found[0]).includes('501'), `the alert names the status: ${textOf(found[0])}`);
    const value = node.children.find((child) => child.props && child.props.className === 'ui-metric-value');
    assert.ok(!value || textOf(value) !== '0', `${label} does not draw 0: ${textOf(node)}`);
  }
",
    );
}

#[test]
fn a_metric_over_an_answered_view_shows_its_value() {
    run(
        "answered",
        r"
  await settle();
  assert.ok(textOf(metric('Items on the shelf')).includes('2'), root.text());
  assert.ok(textOf(metric('Units on the shelf')).includes('95'), root.text());
  assert.equal(alerts(metric('Items on the shelf')).length, 0);
",
    );
}

#[test]
fn a_metric_over_an_answered_empty_view_shows_zero() {
    run(
        "empty",
        r"
  answer = { status: 200, rows: [] };
  await settle();
  assert.ok(textOf(metric('Items on the shelf')).startsWith('0'), root.text());
  assert.equal(alerts(metric('Items on the shelf')).length, 0);
",
    );
}

#[test]
fn a_metric_over_a_forbidden_view_draws_an_alert_naming_the_status() {
    run(
        "forbidden",
        r"
  answer = { status: 403, rows: [] };
  await settle();
  const found = alerts(metric('Items on the shelf'));
  assert.equal(found.length, 1, root.text());
  assert.ok(textOf(found[0]).includes('403'), textOf(found[0]));
",
    );
}

#[test]
fn a_metric_whose_read_is_loading_draws_a_loading_state_not_zero() {
    run(
        "loading",
        r"
  root.settle();
  const node = metric('Items on the shelf');
  const value = node.children.find((child) => child.props && child.props.className === 'ui-metric-value');
  assert.ok(!value || (textOf(value) !== '0' && textOf(value) !== '–'), `loading draws neither 0 nor a dash: ${textOf(node)}`);
  assert.ok(node.props['aria-busy'] === 'true' || node.props['aria-busy'] === true, 'the metric is marked busy');
",
    );
}
