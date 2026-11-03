//! A toggle in the generated React project shows the row state it flips: a row toggle that runs
//! a command is checked from the state `ess_ui::toggle_state` resolves (the `bind` entry written
//! `not <expr>`, or an explicit `checked:`), and a column `as: toggle` is a read-only switch.
//!
//! The generated page runs under `node` against the stand-in `react` of `support`, with a data
//! adapter that answers the list read with three rows and records every command sent.

mod support;

use std::path::{Path, PathBuf};

/// A page listing flags, with the collection's `columns` and `row_actions` lines written in.
fn document(columns: &str, row_actions: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: flags
model: flags.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: flags.list
  sections: [{{name: all, pages: [flags.list]}}]
pages:
  flags.list:
    kind: detail_page
    title: Flags
    sections:
      - name: list
        component: collection
        reads: {{view: flags.All}}
        columns: {columns}
{row_actions}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-toggle-state")
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
const { FlagsListPage } = out('pages/FlagsList');
const rows = [
  { id: 'f-1', title: 'Alpha', enabled: true, locked: false },
  { id: 'f-2', title: 'Beta', enabled: true, locked: true },
  { id: 'f-3', title: 'Gamma', enabled: false, locked: true },
];
const sent = [];
setDataAdapter({
  read: async () => ({ rows: rows.map((row) => ({ ...row })), total: rows.length }),
  send: async (command, input) => { sent.push([command, input]); return { accepted: true }; },
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(FlagsListPage, {}));
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
const checkbox = (node) => node.tag === 'input' && node.props.type === 'checkbox';
// The checkbox of each row toggle, in row order.
const toggles = () => root
  .find((node) => node.tag === 'label' && node.props.className === 'ui-toggle')
  .map((label) => {
    const inputs = inside(label, checkbox);
    assert.equal(inputs.length, 1, 'a toggle holds one checkbox');
    return inputs[0];
  });
";

fn run(test: &str, columns: &str, row_actions: &str, body: &str) {
    let out = scratch(test);
    let yaml = document(columns, row_actions);
    let document =
        ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    support::compile(&out, &["pages/FlagsList.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn a_command_toggle_is_checked_when_the_row_field_is_true() {
    run(
        "derived",
        "[title]",
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}}",
        r"
  await settle();
  assert.ok(root.text().includes('Gamma'), root.text());
  const shown = toggles();
  assert.deepStrictEqual(
    shown.map((input) => input.props.checked),
    [true, true, false],
    'each row toggle shows the row value its bind flips',
  );
  shown[0].props.onChange({ target: { checked: false } });
  for (let i = 0; i < 10; i += 1) await tick();
  assert.deepStrictEqual(sent[0], ['flags.SetFlag', { id: 'f-1', enabled: false }], 'a checked row sends its flipped value');
",
    );
}

#[test]
fn an_explicit_checked_expression_wins() {
    run(
        "explicit",
        "[title]",
        "        row_actions:\n          - {name: enabled, as: toggle, label: Locked, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}, checked: row.locked}",
        r"
  await settle();
  assert.deepStrictEqual(
    toggles().map((input) => input.props.checked),
    [false, true, true],
    'the toggle reads `checked`, not the value its bind flips',
  );
",
    );
}

#[test]
fn a_toggle_column_renders_a_read_only_switch() {
    run(
        "column",
        "[title, {field: enabled, as: toggle, label: Enabled}]",
        "",
        r"
  await settle();
  const cells = root.find((node) => node.tag === 'span' && /\bui-field-toggle\b/.test(node.props.className || ''));
  assert.equal(cells.length, 3, `one switch per row: ${root.text()}`);
  const switches = cells.map((cell) => {
    const inputs = inside(cell, checkbox);
    assert.equal(inputs.length, 1, `the cell is a checkbox: ${JSON.stringify(cell.children)}`);
    return inputs[0];
  });
  assert.deepStrictEqual(switches.map((input) => input.props.checked), [true, true, false], 'the switch reflects the value');
  for (const input of switches) {
    assert.strictEqual(input.props.disabled, true, 'a column switch is read-only');
    assert.strictEqual(input.props['aria-label'], 'Enabled', 'the switch is labelled by its column');
  }
  for (const cell of cells) {
    assert.ok(!/true|false/.test(textOf(cell)), `the cell shows no text value: ${textOf(cell)}`);
  }
",
    );
}
