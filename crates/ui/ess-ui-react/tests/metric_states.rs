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

/// Adversary: the default adapter of a generated project is the fixture adapter, which refuses a
/// view without a fixture with `DataError("no fixture answers <view>", 404)`. The status is on the
/// error, not at the start of its message, and the metric must still name it.
#[test]
fn adversary_a_metric_over_a_view_without_a_fixture_names_404() {
    run(
        "adversary-no-fixture",
        r"
  const { fixtureAdapter } = out('runtime/data');
  setDataAdapter(fixtureAdapter);
  await settle();
  const found = alerts(metric('Items on the shelf'));
  assert.equal(found.length, 1, root.text());
  assert.ok(textOf(found[0]).includes('404'), `the alert names the status 404: ${textOf(found[0])}`);
",
    );
}

/// Adversary: a refresh after an answer keeps the answered value (no loading flash), and a refresh
/// that then fails replaces it with the alert, never with 0.
#[test]
fn adversary_a_refresh_after_an_answer_keeps_the_value_then_fails_to_an_alert() {
    run(
        "adversary-refresh",
        r"
  const { invalidate } = out('runtime/data');
  await settle();
  assert.ok(textOf(metric('Items on the shelf')).startsWith('2'), root.text());
  invalidate();
  root.settle();
  root.settle();
  const node = metric('Items on the shelf');
  assert.notEqual(node.props['aria-busy'], 'true', `a refresh does not flash loading: ${textOf(node)}`);
  assert.ok(textOf(node).startsWith('2'), `a refresh keeps the answered value: ${textOf(node)}`);
  await settle();
  answer = { status: 500, rows: [] };
  invalidate();
  await settle();
  const found = alerts(metric('Items on the shelf'));
  assert.equal(found.length, 1, root.text());
  assert.ok(textOf(found[0]).includes('500'), textOf(found[0]));
  assert.ok(!textOf(metric('Units on the shelf')).startsWith('0'), root.text());
",
    );
}

const FOLDED_DOCUMENT: &str = r"
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
    layout: {columns: [{name: main, frame: shared, title: Shelf, collapsed: true, sections: [list, later]}]}
    sections:
      - {name: list, component: collection, reads: {view: store.shelf.Items}, columns: [qty]}
      - name: later
        title: Later
        collapsed: true
        load: on_visible
        component: collection
        reads: {view: store.shelf.Later}
        columns: [qty]
      - name: settings
        component: form
        does: store.shelf.Save
        groups:
          - {name: advanced, label: Advanced, collapsed: true, fields: [limit]}
";

const FOLDED_HARNESS: &str = r"
browser('/shelf');
const observers = [];
globalThis.IntersectionObserver = class {
  constructor(callback) { this.callback = callback; this.targets = []; observers.push(this); }
  observe(target) { this.targets.push(target); }
  disconnect() {}
};
const { setDataAdapter } = out('runtime/data');
const { ShelfPage } = out('pages/Shelf');
setDataAdapter({ read: async (request) => ({
  rows: [{ qty: request.view === 'store.shelf.Later' ? 4242 : 7 }],
  total: 1,
}) });
const root = React.__root(jsx(ShelfPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const heading = (label) => {
  const found = root.find((node) => node.tag === 'button' && node.props.className === 'ui-collapse' && node.children.join('') === label);
  assert.equal(found.length, 1, `one heading button ${label}: ${root.text()}`);
  return found[0];
};
const byClass = (name) => root.find((node) => node.props && node.props.className === name);
const section = (name) => root.find((node) => node.tag === 'section' && node.props['data-ui-path'] === `pages/shelf/sections/${name}`)[0];
";

fn run_folded(test: &str, body: &str) {
    let out: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-folded")
        .join(test);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(FOLDED_DOCUMENT).expect("the document loads");
    ess_ui_react::generate(&document, Path::new("."), &out).expect("the project generates");
    support::compile(&out, &["pages/Shelf.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "{FOLDED_HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn a_collapsed_frame_and_section_open_on_a_click_and_the_read_starts_when_the_section_opens() {
    run_folded(
        "frame-and-section",
        r"
  await settle();
  assert.equal(heading('Shelf').props['aria-expanded'], false, 'the frame starts closed');
  assert.equal(byClass('ui-frame-body')[0].props.hidden, true, 'a closed frame hides its body');
  assert.ok(section('list'), 'a closed frame keeps its sections mounted');
  click(heading('Shelf'));
  await settle();
  assert.equal(heading('Shelf').props['aria-expanded'], true);
  assert.equal(byClass('ui-frame-body')[0].props.hidden, false);
  assert.equal(heading('Later').props['aria-expanded'], false, 'the section starts closed inside the open frame');
  section('later').props.ref.current = {};
  await settle();
  assert.equal(observers.length, 0, 'a closed section starts no read');
  assert.ok(!root.text().includes('4,242'), root.text());
  click(heading('Later'));
  await settle();
  assert.equal(heading('Later').props['aria-expanded'], true);
  assert.ok(observers.length > 0, 'an opened on_visible section watches for its view');
  observers[observers.length - 1].callback([{ isIntersecting: true }]);
  await settle();
  assert.ok(root.text().includes('4,242'), `the opened section read its rows: ${root.text()}`);
",
    );
}

#[test]
fn a_collapsed_form_group_keeps_its_fields_mounted_so_they_are_still_submitted() {
    run_folded(
        "form-group",
        r"
  await settle();
  assert.equal(heading('Advanced').props['aria-expanded'], false, 'the group starts closed');
  const body = root.find((node) => node.tag === 'fieldset')[0].children.find((child) => child.props && child.props.className === 'ui-collapse-body');
  assert.equal(body.props.hidden, true);
  assert.ok(root.find((node) => node.tag === 'input' || node.tag === 'textarea' || node.tag === 'select').length > 0, `the closed group's field is still in the form: ${root.text()}`);
  click(heading('Advanced'));
  await settle();
  assert.equal(heading('Advanced').props['aria-expanded'], true);
",
    );
}
