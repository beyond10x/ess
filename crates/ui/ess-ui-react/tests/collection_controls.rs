//! A collection in the generated React project draws the controls it declares
//! (beyond10x/ess#723): numeric sort, an `Actions` heading, a table that scrolls inside its
//! frame, card actions beside or below the fields, a sort control on cards and lists, the expand
//! control, the drag handle, select-all with a bulk menu, item and toggle columns with their own
//! heading, and the default subset of selectable columns.
//!
//! The generated page runs under `node` against the stand-in `react` of `support`, with a data
//! adapter that answers the list read with three rows and records every command sent. Layout
//! properties are read from the generated stylesheet.

mod support;

use std::path::{Path, PathBuf};

/// A page listing items: `state` holds the page's `state:` lines (empty for none), `collection`
/// the collection's lines after its `reads`.
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
        .join("ess-ui-react-collection-controls")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// Shared by every case: three rows, an adapter recording what is sent, the page and helpers.
const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { ShelfItemsPage } = out('pages/ShelfItems');
const rows = [
  { id: 'a', title: 'Alpha', size: 5, enabled: true, owner: 'ann' },
  { id: 'b', title: 'Beta', size: 10, enabled: false, owner: 'bob' },
  { id: 'c', title: 'Gamma', size: 1, enabled: true, owner: 'cy' },
];
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
const checkbox = (node) => node.tag === 'input' && node.props.type === 'checkbox';
const elements = (node) => (node.children || []).filter((child) => typeof child === 'object');
// The rendered rows (`tr` or `li`), in order.
const rowNodes = () => root.find((node) => (node.tag === 'tr' || node.tag === 'li') && hasClass('ui-row')(node));
// The title each row shows, in row order.
const titles = () => rowNodes().map((row) => ['Alpha', 'Beta', 'Gamma'].find((title) => textOf(row).includes(title)));
const cells = (row) => elements(row).filter((node) => node.tag === 'td');
const headings = () => tagged('th').map((th) => textOf(th).replace(/[▲▼]/g, '').trim());
const rowOf = (title) => rowNodes().find((row) => textOf(row).includes(title));
const event = (extra) => ({ prevented: false, preventDefault() { this.prevented = true; }, stopPropagation() {}, ...extra });
";

fn generate(test: &str, state: &str, collection: &str) -> PathBuf {
    let out = scratch(test);
    let yaml = document(state, collection);
    let document =
        ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    support::compile(&out, &["pages/ShelfItems.tsx"]);
    out
}

fn script(out: &Path, test: &str, body: &str) {
    support::node(
        out,
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

fn run(test: &str, state: &str, collection: &str, body: &str) -> PathBuf {
    let out = generate(test, state, collection);
    script(&out, test, body);
    out
}

/// The generated stylesheet's rule lines whose selector names `class`.
fn rules(out: &Path, class: &str) -> Vec<String> {
    let css = std::fs::read_to_string(out.join("src/runtime/styles.css")).expect("the styles");
    css.lines()
        .filter(|line| {
            line.split('{')
                .next()
                .is_some_and(|selector| selector.contains(class))
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_number_column_sorts_by_value() {
    run(
        "number",
        "",
        "        columns: [title, {field: size, as: number, sortable: true}]\n        sort: {by: size}",
        r"
  await settle();
  assert.deepStrictEqual(titles(), ['Gamma', 'Alpha', 'Beta'], `sizes 1, 5, 10 ascending: ${root.text()}`);
  const [sort] = root.find((node) => node.tag === 'button' && hasClass('ui-sort')(node) && textOf(node).includes('size'));
  assert.ok(sort, 'the size column sorts');
  click(sort);
  await settle();
  assert.deepStrictEqual(titles(), ['Beta', 'Alpha', 'Gamma'], 'sizes 10, 5, 1 descending');
",
    );
}

#[test]
fn the_row_actions_column_has_a_header() {
    run(
        "actions-header",
        "",
        "        columns: [title]\n        row_actions: [{name: archive, does: shelf.Archive, label: Archive}]",
        r"
  await settle();
  assert.deepStrictEqual(headings(), ['title', 'Actions'], 'the actions column is headed');
  for (const row of rowNodes()) {
    const tds = cells(row);
    assert.equal(tds.length, headings().length, 'every cell has its heading');
    assert.ok(inside(tds[tds.length - 1], (node) => node.tag === 'button' && node.props['aria-label'] === 'Archive').length === 1, 'the last cell holds the row action');
  }
",
    );
}

#[test]
fn a_wide_table_scrolls_inside_its_frame() {
    let out = run(
        "wide",
        "",
        "        columns: [title, size, owner]\n        row_actions: [{name: archive, does: shelf.Archive, label: Archive}]",
        r"
  await settle();
  const frames = classed('ui-table-frame');
  assert.equal(frames.length, 1, 'the table sits in a frame');
  assert.equal(inside(frames[0], (node) => node.tag === 'table').length, 1, 'the frame holds the table');
  for (const row of rowNodes()) {
    const tds = cells(row);
    assert.equal(tds.length, 4, 'three declared columns and the actions cell');
    const holding = tds.filter((td) => inside(td, (node) => node.tag === 'button' && node.props['aria-label'] === 'Archive').length > 0);
    assert.deepStrictEqual(holding, [tds[3]], 'only the actions cell holds the row action');
    assert.ok(hasClass('ui-actions-cell')(tds[3]), 'the actions cell is marked');
  }
",
    );
    let frame = rules(&out, ".ui-table-frame");
    assert!(
        frame.iter().any(|rule| rule.contains("overflow-x: auto")),
        "the frame scrolls: {frame:?}"
    );
    assert!(
        frame
            .iter()
            .any(|rule| rule.contains(" local") && rule.contains(" scroll")),
        "the frame shades the side that scrolls: {frame:?}"
    );
    for class in [".ui-actions-cell", ".ui-actions-head", ".ui-row-actions"] {
        for rule in rules(&out, class) {
            assert!(
                !rule.contains("position: sticky") && !rule.contains("position: absolute"),
                "the actions cell lies over no column: {rule}"
            );
        }
    }
}

#[test]
fn card_actions_leave_the_title_whole() {
    let out = run(
        "cards",
        "",
        "        style: cards
        columns: [title, {field: enabled, as: toggle, label: Enabled}]
        row_actions:
          - {name: archive, does: shelf.Archive, label: Archive}
          - {name: on, as: toggle, label: On, does: shelf.SetOn, bind: {id: row.id, enabled: not row.enabled}}
        item:
          - {name: owner, primitive: text, text: row.owner}",
        r"
  await settle();
  const cards = rowNodes();
  assert.equal(cards.length, 3, root.text());
  for (const card of cards) {
    const parts = elements(card);
    const title = parts.findIndex((part) => /Alpha|Beta|Gamma/.test(textOf(part)));
    const actions = parts.findIndex((part) => inside(part, (node) => node.tag === 'button' && node.props['aria-label'] === 'Archive').length > 0);
    assert.ok(title >= 0 && actions > title, `the actions follow the title: ${JSON.stringify(parts.map(textOf))}`);
    assert.ok(!/Alpha|Beta|Gamma/.test(textOf(parts[actions])), 'the actions share no box with the title');
    assert.ok(/ann|bob|cy/.test(textOf(card)), 'the item is drawn beside the fields');
  }
  const switches = classed('ui-field-toggle').map((cell) => inside(cell, checkbox)[0]);
  assert.deepStrictEqual(switches.map((input) => input && input.props.checked), [true, false, true], 'the toggle column is drawn in its state');
  const toggles = root.find((node) => node.tag === 'label' && node.props.className === 'ui-toggle').map((label) => inside(label, checkbox)[0]);
  assert.deepStrictEqual(toggles.map((input) => input.props.checked), [true, false, true], 'the toggle action is drawn in its state');
",
    );
    let card = rules(&out, ".ui-card");
    assert!(
        card.iter()
            .any(|rule| rule.starts_with(".ui-card ") && rule.contains("flex-direction: column")),
        "a card stacks its fields and actions: {card:?}"
    );
    for class in [".ui-row-actions", ".ui-row-tail"] {
        for rule in rules(&out, class) {
            assert!(
                !rule.contains("position: absolute"),
                "card actions lie over nothing: {rule}"
            );
        }
    }
}

#[test]
fn cards_and_lists_draw_their_declared_sort() {
    for style in ["cards", "list"] {
        run(
            &format!("sort-{style}"),
            "",
            &format!(
                "        style: {style}\n        columns: [title, {{field: size, as: number}}]\n        sort: {{by: size, allowed: [title, size]}}"
            ),
            r"
  await settle();
  const [select] = root.find((node) => node.tag === 'select' && node.props['aria-label'] === 'Sort by');
  assert.ok(select, `a sort control is drawn: ${root.text()}`);
  assert.deepStrictEqual(elements(select).map((option) => option.props.value), ['title', 'size'], 'it offers the allowed fields');
  assert.equal(select.props.value, 'size', 'it shows the declared sort');
  assert.deepStrictEqual(titles(), ['Gamma', 'Alpha', 'Beta'], 'rows follow the declared sort');
  select.props.onChange({ target: { value: 'title' } });
  await settle();
  assert.deepStrictEqual(titles(), ['Alpha', 'Beta', 'Gamma'], 'choosing a field sorts by it');
  const [direction] = classed('ui-sort-dir');
  click(direction);
  await settle();
  assert.deepStrictEqual(titles(), ['Gamma', 'Beta', 'Alpha'], 'the direction control reverses it');
",
        );
    }
}

#[test]
fn expand_reorder_and_bulk_actions_draw_their_controls() {
    run(
        "controls",
        "",
        "        columns: [title]
        selection: multiple
        bulk_actions: [{name: archive, does: shelf.ArchiveAll, label: Archive, bind: {ids: selection.id}}]
        reorder: {does: shelf.Move}
        expand: {name: detail, primitive: text, text: row.owner}",
        r"
  await settle();
  // Expand: one control per row; it shows that row's detail and hides it again.
  const expanders = () => root.find((node) => node.tag === 'button' && hasClass('ui-expand-toggle')(node));
  assert.equal(expanders().length, 3, `an expand control per row: ${root.text()}`);
  click(expanders()[0]);
  await settle();
  assert.deepStrictEqual(classed('ui-expand').map(textOf), ['ann'], 'the first row expands');
  assert.equal(expanders()[0].props['aria-expanded'], true, 'its control says so');
  click(expanders()[0]);
  await settle();
  assert.equal(classed('ui-expand').length, 0, 'it collapses again');
  // Reorder: a drag handle per row; a drop on a neighbour and an arrow key send `{id, delta}`.
  const handles = () => classed('ui-drag-handle');
  assert.equal(handles().length, 3, 'a drag handle per row');
  assert.ok(handles().every((handle) => handle.props.draggable === true), 'each handle drags');
  handles()[0].props.onDragStart({ dataTransfer: { setData() {} } });
  const far = event();
  rowOf('Gamma').props.onDragOver(far);
  assert.equal(far.prevented, false, 'a drop two rows away is refused');
  const near = event();
  rowOf('Beta').props.onDragOver(near);
  assert.equal(near.prevented, true, 'a drop on the next row is taken');
  rowOf('Beta').props.onDrop(event());
  handles()[2].props.onKeyDown(event({ key: 'ArrowUp' }));
  for (let i = 0; i < 10; i += 1) await tick();
  assert.deepStrictEqual(sent, [['shelf.Move', { id: 'a', delta: 1 }], ['shelf.Move', { id: 'c', delta: -1 }]], 'each move sends its row and one place');
  sent.length = 0;
  // Select-all and the bulk menu.
  const [all] = root.find((node) => node.tag === 'input' && node.props['aria-label'] === 'Select all rows');
  assert.ok(all, 'a select-all is drawn');
  assert.equal(classed('ui-bulk-menu').length, 0, 'no bulk menu before a selection');
  all.props.onChange({ target: { checked: true } });
  await settle();
  const boxes = root.find((node) => checkbox(node) && node.props['aria-label'] === 'Select row');
  assert.deepStrictEqual(boxes.map((box) => box.props.checked), [true, true, true], 'every row is selected');
  const [menu] = classed('ui-bulk-menu');
  assert.ok(menu && textOf(menu).includes('3 selected'), `the bulk menu counts the selection: ${menu && textOf(menu)}`);
  const [archive] = inside(menu, (node) => node.tag === 'button' && node.props['aria-label'] === 'Archive');
  click(archive);
  for (let i = 0; i < 10; i += 1) await tick();
  assert.deepStrictEqual(sent, [['shelf.ArchiveAll', { ids: ['a', 'b', 'c'] }]], 'the bulk action sends the selection');
  root.find((node) => node.tag === 'input' && node.props['aria-label'] === 'Select all rows')[0].props.onChange({ target: { checked: false } });
  await settle();
  assert.equal(classed('ui-bulk-menu').length, 0, 'clearing the selection closes the bulk menu');
",
    );
}

#[test]
fn a_row_item_and_a_toggle_column_get_their_own_column() {
    run(
        "item-toggle",
        "",
        "        columns: [title]
        sort: {by: title, allowed: [title, owner, enabled]}
        row_actions:
          - {name: archive, does: shelf.Archive, label: Archive}
          - {name: on, as: toggle, label: Enabled, does: shelf.SetOn, bind: {id: row.id, enabled: not row.enabled}}
        item:
          - {name: owner, primitive: text, text: row.owner}",
        r"
  await settle();
  assert.deepStrictEqual(headings(), ['title', 'owner', 'Enabled', 'Actions'], 'the item and the toggle are headed');
  for (const row of rowNodes()) {
    const tds = cells(row);
    assert.equal(tds.length, 4, 'one cell per heading');
    assert.ok(/^(ann|bob|cy)$/.test(textOf(tds[1])), `the item has its own cell: ${textOf(tds[1])}`);
    assert.equal(inside(tds[2], checkbox).length, 1, 'the toggle has its own cell');
    assert.equal(inside(tds[3], checkbox).length, 0, 'the actions cell holds no toggle');
    assert.ok(!/ann|bob|cy/.test(textOf(tds[3])), 'the actions cell holds no item');
  }
  assert.deepStrictEqual(
    rowNodes().map((row) => inside(cells(row)[2], checkbox)[0].props.checked),
    [true, false, true],
    'the toggle column shows each row state',
  );
  const sortBy = (heading) => {
    const [th] = tagged('th').filter((node) => textOf(node).replace(/[▲▼]/g, '').trim() === heading);
    const [button] = inside(th, (node) => node.tag === 'button' && hasClass('ui-sort')(node));
    assert.ok(button, `the ${heading} column sorts`);
    click(button);
  };
  sortBy('Enabled');
  await settle();
  assert.deepStrictEqual(titles(), ['Beta', 'Alpha', 'Gamma'], 'off before on');
  sortBy('owner');
  await settle();
  sortBy('owner');
  await settle();
  assert.deepStrictEqual(titles(), ['Gamma', 'Beta', 'Alpha'], 'by the item field, descending');
",
    );
}

#[test]
fn a_default_column_subset_renders_only_its_columns() {
    run(
        "subset",
        "    state:\n      columns: {type: {list: string}, class: component_state, default: [title, size]}\n",
        "        columns: {binds: state.columns, all: [title, size, owner]}",
        r"
  await settle();
  assert.deepStrictEqual(headings(), ['title', 'size'], 'only the default columns are headed');
  for (const row of rowNodes()) {
    assert.equal(cells(row).length, 2, 'only the default columns are drawn');
  }
  assert.ok(!/ann|bob|cy/.test(root.text()), `the owner column is not drawn: ${root.text()}`);
",
    );
}
