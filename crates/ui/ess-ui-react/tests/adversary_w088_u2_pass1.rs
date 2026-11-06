//! Adversary pass 1 on the collection controls of beyond10x/ess#723 (wave 0.88.0, unit 2).
//!
//! Each case generates a page holding one collection, runs it under `node` against the stand-in
//! `react` of `support` with a data adapter that answers the list read with the case's rows and
//! records every command sent, and asserts what a user of the drawn controls would see or send.

mod support;

use std::path::{Path, PathBuf};

fn document(state: &str, collection: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: shelf
model: shelf.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: shelf.items
  sections: [{{name: all, pages: [shelf.items]}}]
pages:
  shelf.items:
    kind: detail_page
    title: Items
{state}    sections:
      - name: list
        component: collection
        reads: {{view: shelf.Items}}
{collection}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w088-u2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// The rows the adapter answers with: `id`, `title`, `size`, `stage`, `owner`.
const ROWS: &str = r"[
  { id: 'a', title: 'Alpha', size: 5, stage: 'open', owner: 'ann' },
  { id: 'b', title: 'Beta', size: 10, stage: 'won', owner: 'bob' },
  { id: 'c', title: 'Gamma', size: 1, stage: 'open', owner: 'cy' },
]";

const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { ShelfItemsPage } = out('pages/ShelfItems');
const sent = [];
setDataAdapter({
  read: async () => ({ rows: rows.map((row) => ({ ...row })), total: rows.length }),
  send: async (command, input) => { sent.push([command, input]); return { accepted: true }; },
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(ShelfItemsPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
async function drain() { for (let i = 0; i < 10; i += 1) await tick(); root.settle(); }
function textOf(node) {
  if (typeof node === 'string') return node;
  return (node.children || []).map(textOf).join('');
}
const inside = (node, predicate) => {
  const found = [];
  const visit = (n) => {
    if (typeof n !== 'object' || n === null) return;
    if (predicate(n)) found.push(n);
    (n.children || []).forEach(visit);
  };
  (node.children || []).forEach(visit);
  return found;
};
const hasClass = (name) => (node) => typeof node.props.className === 'string' && node.props.className.split(' ').includes(name);
const classed = (name) => root.find(hasClass(name));
const tagged = (tag) => root.find((node) => node.tag === tag);
const rowNodes = () => root.find((node) => (node.tag === 'tr' || node.tag === 'li') && hasClass('ui-row')(node));
const titleOf = (row) => rows.map((r) => r.title).find((title) => textOf(row).includes(title));
const titles = () => rowNodes().map(titleOf);
const rowOf = (title) => rowNodes().find((row) => textOf(row).includes(title));
const headings = () => tagged('th').map((th) => textOf(th).replace(/[▲▼]/g, '').trim());
const cells = (row) => (row.children || []).filter((node) => typeof node === 'object' && node.tag === 'td');
const handleOf = (title) => inside(rowOf(title), hasClass('ui-drag-handle'))[0];
const rowBox = (row) => inside(row, (node) => node.tag === 'input' && node.props['aria-label'] === 'Select row')[0];
// An event that records whether a handler stopped it or prevented its default.
const event = (extra) => ({
  stopped: false, prevented: false,
  stopPropagation() { this.stopped = true; }, preventDefault() { this.prevented = true; },
  ...extra,
});
";

fn run(test: &str, rows: &str, state: &str, collection: &str, body: &str) {
    let out = scratch(test);
    let yaml = document(state, collection);
    let document =
        ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    support::compile(&out, &["pages/ShelfItems.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "const rows = {rows};\n{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// The drag handle's arrow keys move a row one place; the first row has no place above it and the
/// last none below, so those keys send nothing (a drop is refused the same way when it is not on a
/// neighbour).
#[test]
fn ad_arrow_keys_on_the_first_and_last_handle_send_no_move() {
    run(
        "edge-keys",
        ROWS,
        "",
        "        columns: [title]\n        reorder: {does: shelf.Move}",
        r"
  await settle();
  assert.deepStrictEqual(titles(), ['Alpha', 'Beta', 'Gamma'], root.text());
  handleOf('Alpha').props.onKeyDown(event({ key: 'ArrowUp' }));
  handleOf('Gamma').props.onKeyDown(event({ key: 'ArrowDown' }));
  await drain();
  assert.deepStrictEqual(sent, [], 'no row moves past either end of the list');
",
    );
}

/// The kanban the schema documents (`group_by` and `reorder`, as the partner-portal example's
/// pipeline page writes it): a drag is taken only onto the row shown next to the dragged one in
/// its own group, and a card of another group is not a neighbour.
#[test]
fn ad_a_kanban_drag_follows_the_rows_as_grouped() {
    run(
        "kanban",
        ROWS,
        "",
        "        style: cards\n        group_by: stage\n        columns: [title]\n        reorder: {does: shelf.Move}\n        degrades: {no_drag: move_buttons}",
        r"
  await settle();
  // Groups: open holds Alpha then Gamma; won holds Beta.
  assert.deepStrictEqual(titles(), ['Alpha', 'Gamma', 'Beta'], root.text());
  handleOf('Alpha').props.onDragStart({ dataTransfer: { setData() {} } });
  const across = event();
  rowOf('Beta').props.onDragOver(across);
  rowOf('Beta').props.onDrop(event());
  await drain();
  assert.equal(across.prevented, false, 'a card in another group is not a neighbour');
  assert.deepStrictEqual(sent, [], `a drop onto another group sends no move: ${JSON.stringify(sent)}`);
  handleOf('Alpha').props.onDragStart({ dataTransfer: { setData() {} } });
  const next = event();
  rowOf('Gamma').props.onDragOver(next);
  assert.equal(next.prevented, true, 'the card shown next in the same group takes the drop');
",
    );
}

/// A table grouped by a field draws one table per group; a select-all selects the rows of the
/// table it heads, never a row drawn under another group's heading.
#[test]
fn ad_a_select_all_selects_only_the_rows_below_it() {
    run(
        "group-select-all",
        ROWS,
        "",
        "        group_by: stage\n        selection: multiple\n        columns: [title]\n        bulk_actions: [{name: archive, does: shelf.ArchiveAll, label: Archive, bind: {ids: selection.id}}]",
        r"
  await settle();
  const boxes = () => root.find((node) => node.tag === 'input' && node.props['aria-label'] === 'Select all rows');
  assert.ok(boxes().length >= 1, `a select-all is drawn: ${root.text()}`);
  const first = boxes()[0];
  const table = tagged('table').find((t) => inside(t, (node) => node === first).length > 0);
  const below = table ? inside(table, (node) => node.tag === 'tr' && hasClass('ui-row')(node)).map(titleOf) : titles();
  first.props.onChange({ target: { checked: true } });
  await settle();
  const chosen = rowNodes().filter((row) => rowBox(row).props.checked).map(titleOf).sort();
  assert.deepStrictEqual(chosen, [...below].sort(), `the select-all heading ${JSON.stringify(below)} selects only those rows`);
  const [menu] = classed('ui-bulk-menu');
  assert.ok(menu && textOf(menu).includes(`${below.length} selected`), `the bulk menu counts them: ${menu && textOf(menu)}`);
",
    );
}

/// With a declared sort the rows are shown in another order than the read's; a drag between two
/// rows shown next to each other must not send a move the read's order cannot make: Gamma is the
/// read's last row, so "one place down" is past its end.
#[test]
fn ad_a_drag_under_a_client_sort_sends_no_move_past_the_reads_end() {
    run(
        "sorted-drag",
        ROWS,
        "",
        "        columns: [title, {field: size, as: number}]\n        sort: {by: size}\n        reorder: {does: shelf.Move}",
        r"
  await settle();
  assert.deepStrictEqual(titles(), ['Gamma', 'Alpha', 'Beta'], 'shown by size');
  handleOf('Gamma').props.onDragStart({ dataTransfer: { setData() {} } });
  rowOf('Alpha').props.onDrop(event());
  handleOf('Gamma').props.onKeyDown(event({ key: 'ArrowDown' }));
  await drain();
  const past = sent.filter(([, input]) => input && input.id === 'c' && input.delta === 1);
  assert.deepStrictEqual(past, [], `the read's last row is not moved down: ${JSON.stringify(sent)}`);
",
    );
}

/// A click on the drag handle is not a click on the row: it leaves an expandable row as it was.
#[test]
fn ad_a_click_on_the_drag_handle_does_not_expand_the_row() {
    run(
        "handle-click",
        ROWS,
        "",
        "        columns: [title]\n        reorder: {does: shelf.Move}\n        expand: {name: detail, primitive: text, text: row.owner}",
        r"
  await settle();
  const handle = handleOf('Alpha');
  // The browser delivers the click to the handle, then to the row unless a handler stops it.
  const click = event({ button: 0 });
  if (handle.props.onClick) handle.props.onClick(click);
  if (!click.stopped) rowOf('Alpha').props.onClick(click);
  await settle();
  assert.equal(classed('ui-expand').length, 0, `the handle click expands nothing: ${classed('ui-expand').map(textOf)}`);
",
    );
}

/// `as: number` values given as text, with negatives, decimals and a missing value: the numbers
/// sort by value.
#[test]
fn ad_numeric_text_with_negatives_decimals_and_a_null_sorts_by_value() {
    run(
        "numeric-text",
        r"[
  { id: 'a', title: 'Alpha', size: '10' },
  { id: 'b', title: 'Beta', size: '-2.5' },
  { id: 'c', title: 'Gamma', size: null },
  { id: 'd', title: 'Delta', size: '0.5' },
  { id: 'e', title: 'Epsilon', size: '3' },
]",
        "",
        "        columns: [title, {field: size, as: number, sortable: true}]\n        sort: {by: size}",
        r"
  await settle();
  const shown = titles().filter((title) => title !== 'Gamma');
  assert.deepStrictEqual(shown, ['Beta', 'Delta', 'Epsilon', 'Alpha'], `-2.5, 0.5, 3, 10: ${titles()}`);
",
    );
}

/// One numeric column holding a number, a number as text and a placeholder: the two numbers
/// still sort by value, whatever order the read returns them in.
#[test]
fn ad_numeric_sort_orders_the_numbers_whatever_else_the_column_holds() {
    run(
        "numeric-mixed",
        r"[
  { id: 'a', title: 'Alpha', size: '10' },
  { id: 'b', title: 'Beta', size: 'n/a' },
  { id: 'c', title: 'Gamma', size: 5 },
]",
        "",
        "        columns: [title, {field: size, as: number, sortable: true}]\n        sort: {by: size}",
        r"
  await settle();
  const shown = titles().filter((title) => title !== 'Beta');
  assert.deepStrictEqual(shown, ['Gamma', 'Alpha'], `5 before 10: ${titles()}`);
",
    );
}

/// A default subset naming a column the collection does not declare keeps the declared ones it
/// names; a collection without row actions draws no `Actions` heading; toggles alone draw their
/// own column and no `Actions` column.
#[test]
fn ad_unknown_subset_names_and_headings_without_row_buttons() {
    run(
        "subset-unknown",
        ROWS,
        "    state:\n      columns: {type: {list: string}, class: component_state, default: [title, missing]}\n",
        "        columns: {binds: state.columns, all: [title, size, owner]}\n        row_actions:\n          - {name: on, as: toggle, label: On, does: shelf.SetOn, bind: {id: row.id}}",
        r"
  await settle();
  assert.deepStrictEqual(headings(), ['title', 'On'], `the known column and the toggle: ${headings()}`);
  for (const row of rowNodes()) {
    assert.equal(cells(row).length, 2, 'one cell per heading');
  }
",
    );
}
