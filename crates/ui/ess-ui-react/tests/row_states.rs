//! A row condition is judged per row, a read that has not started shows its empty state, and a
//! board row without a widget names why (story:each-row-and-each-waiting-read-draws-what-the-document-declares,
//! beyond10x/ess#723).
//!
//! The generated page runs under `node` against the stand-in `react` of `support`, with a data
//! adapter that answers each view from a table.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DOCUMENT: &str = r"
format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders
  sections: [{name: all, pages: [orders]}]
pages:
  orders:
    kind: list_page
    title: Orders
    sections:
      - name: table
        component: collection
        reads: {view: desk.Orders}
        columns:
          - id
          - {field: note, visible: row.has_note}
          - {field: secret, visible: params.reveal}
      - name: cards
        component: collection
        style: cards
        reads: {view: desk.Orders}
        columns:
          - id
          - {field: note, visible: row.has_note}
          - {field: secret, visible: params.reveal}
      - name: noted
        component: record
        reads: {view: desk.Orders}
        fields:
          - id
          - {field: note, visible: row.has_note}
          - {field: secret, visible: params.reveal}
      - name: quiet
        component: record
        reads: {view: desk.Quiet}
        fields:
          - id
          - {field: note, visible: row.has_note}
      - name: pick
        component: collection
        selection: single
        reads: {view: desk.Orders}
        columns: [id]
      - name: detail
        component: record
        depends_on: pick
        reads: {view: desk.Detail, params: {id: section.pick.selection.id}}
        fields: [id]
        states:
          empty:
            title: No order
            message: Pick an order
            action: {name: reload, label: Reload, navigate: {to: orders}}
      - name: bare
        component: record
        depends_on: pick
        reads: {view: desk.Detail, params: {id: section.pick.selection.id}}
        fields: [id]
      - name: slow
        component: record
        reads: {view: desk.Slow}
        fields: [id]
        states: {empty: {message: Nothing slow}}
      - name: board
        component: board
        reads: {view: desk.Widgets}
        widget_by: kind
        widgets:
          gauge: {primitive: text, text: row.id}
";

const HARNESS: &str = r"
browser('/orders');
const { setDataAdapter } = out('runtime/data');
const { OrdersPage } = out('pages/Orders');
const tables = {
  'desk.Orders': [
    { id: 'o1', note: 'fragile', has_note: true, secret: 'x' },
    { id: 'o2', note: 'stale', has_note: false, secret: 'y' },
  ],
  'desk.Quiet': [{ id: 'q1', note: 'unseen', has_note: false }],
  'desk.Detail': [],
  'desk.Widgets': [
    { widget: 'w1', id: 'w1', kind: 'gauge', value: 1 },
    { widget: 'w2', id: 'w2', value: 7 },
    { widget: 'w3', id: 'w3', kind: 'ghost', value: 3 },
  ],
};
setDataAdapter({ read: (request) => {
  if (request.view === 'desk.Slow') return new Promise(() => {});
  const rows = tables[request.view] || [];
  return Promise.resolve({ rows: rows.map((row) => ({ ...row })), total: rows.length });
}});
const root = React.__root(jsx(OrdersPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === 'string') return node;
  return (node.children || []).map(textOf).join('');
}
const section = (name) => {
  const found = root.find((node) => node.tag === 'section' && String(node.props['data-ui-path']).endsWith('/sections/' + name));
  assert.equal(found.length, 1, `one section ${name}: ${root.text()}`);
  return found[0];
};
const within = (node, test) => {
  const found = [];
  const walk = (entry) => {
    if (typeof entry === 'string') return;
    if (test(entry)) found.push(entry);
    (entry.children || []).forEach(walk);
  };
  walk(node);
  return found;
};
const tagged = (node, tag) => within(node, (entry) => entry.tag === tag);
const classed = (node, name) => within(node, (entry) => String(entry.props && entry.props.className || '').split(' ').includes(name));
";

static PROJECT: OnceLock<PathBuf> = OnceLock::new();

fn project() -> &'static Path {
    PROJECT.get_or_init(|| {
        let out: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-react-row-states");
        if out.exists() {
            std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
        }
        let document = ess_ui::load_str(DOCUMENT).expect("the document loads");
        ess_ui_react::generate(&document, Path::new("."), &out).expect("the project generates");
        support::compile(&out, &["pages/Orders.tsx"]);
        out
    })
}

fn run(test: &str, body: &str) {
    support::node(
        project(),
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn a_column_condition_reading_the_row_is_judged_per_row() {
    run(
        "per_row",
        r"
  await settle();
  const table = section('table');
  const heads = tagged(table, 'th').map(textOf);
  assert.ok(heads.some((head) => /note/i.test(head)), `the note header stays: ${heads}`);
  assert.ok(!heads.some((head) => /secret/i.test(head)), `a column that reads no row is hidden whole: ${heads}`);
  const rows = tagged(tagged(table, 'tbody')[0], 'tr');
  assert.equal(rows.length, 2, textOf(table));
  const cells = (row) => tagged(row, 'td').map(textOf);
  assert.deepEqual(cells(rows[0]), ['o1', 'fragile']);
  assert.deepEqual(cells(rows[1]), ['o2', ''], 'the cell of a row where the condition is false is empty');

  const cards = classed(section('cards'), 'ui-card');
  assert.equal(cards.length, 2, textOf(section('cards')));
  const labels = (card) => tagged(card, 'dt').map(textOf);
  assert.equal(labels(cards[0]).length, 2, `the first card draws id and note: ${textOf(cards[0])}`);
  assert.equal(labels(cards[1]).length, 1, `the second card omits note: ${textOf(cards[1])}`);
  assert.ok(textOf(cards[0]).includes('fragile'));
  assert.ok(!textOf(cards[1]).includes('secret'));

  const noted = tagged(section('noted'), 'dt').map(textOf);
  assert.equal(noted.length, 2, `a record whose row has a note draws it: ${noted}`);
  const quiet = tagged(section('quiet'), 'dt').map(textOf);
  assert.equal(quiet.length, 1, `a record whose row has none omits it: ${quiet}`);
",
    );
}

#[test]
fn a_read_that_has_not_started_shows_its_empty_state() {
    run(
        "idle",
        r"
  await settle();
  const detail = section('detail');
  assert.equal(classed(detail, 'ui-skeleton').length, 0, `no skeleton: ${textOf(detail)}`);
  const empty = classed(detail, 'ui-state-empty');
  assert.equal(empty.length, 1, `the declared empty state: ${textOf(detail)}`);
  assert.equal(textOf(tagged(empty[0], 'h3')[0]), 'No order');
  assert.equal(textOf(tagged(empty[0], 'p')[0]), 'Pick an order');
  assert.ok(textOf(empty[0]).includes('Reload'), `the empty action: ${textOf(empty[0])}`);
  const bare = section('bare');
  assert.equal(classed(bare, 'ui-skeleton').length, 0, `no skeleton: ${textOf(bare)}`);
  assert.ok(textOf(bare).includes('Nothing to show yet'), textOf(bare));
  assert.equal(classed(section('slow'), 'ui-skeleton').length, 1, 'a read that is loading still draws the skeleton');
",
    );
}

#[test]
fn a_board_row_without_a_widget_names_its_kind() {
    run(
        "board",
        r"
  await settle();
  const board = section('board');
  const items = classed(board, 'ui-board-item');
  assert.equal(items.length, 3, textOf(board));
  const shown = items.map(textOf);
  assert.ok(shown[0].includes('w1') && !shown[0].includes('No widget'), shown[0]);
  assert.ok(shown[1].includes('no kind'), `a row without a kind names the field: ${shown[1]}`);
  assert.ok(shown[2].includes('No widget for ghost'), `a kind no widget names: ${shown[2]}`);
  assert.ok(!textOf(board).includes('undefined'), textOf(board));
",
    );
}
