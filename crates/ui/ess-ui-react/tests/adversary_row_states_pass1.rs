//! Adversary pass 1 on story:each-row-and-each-waiting-read-draws-what-the-document-declares:
//! the React half of the cases where React and the terminal must draw the same column for the same
//! `visible:` (the terminal half is `ess-ui-tui/tests/adversary_row_states_pass1.rs`).
//!
//! The schema: "A condition that reads `row.<path>` is judged for each row ...; any other condition
//! shows or hides the whole column". A condition whose only `row.` is inside a quoted string, or is
//! the tail of a hyphenated name (`params.show-row.on`), reads no row path.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DOCUMENT: &str = r#"
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
          - {field: note, visible: row.has_note or params.reveal}
          - {field: secret, visible: "params.mode == 'row.x'"}
          - {field: code, visible: params.show-row.on}
"#;

const HARNESS: &str = r"
browser('/orders');
const { setDataAdapter } = out('runtime/data');
const { OrdersPage } = out('pages/Orders');
const tables = {
  'desk.Orders': [
    { id: 'o1', note: 'fragile', has_note: true, secret: 'sx', code: 'c1' },
    { id: 'o2', note: 'stale', has_note: false, secret: 'sy', code: 'c2' },
  ],
};
setDataAdapter({ read: (request) => {
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
";

static PROJECT: OnceLock<PathBuf> = OnceLock::new();

fn project() -> &'static Path {
    PROJECT.get_or_init(|| {
        let out: PathBuf =
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-react-adversary-row-states-pass1");
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

/// `params.mode == 'row.x'` reads `params.mode` and compares it with a text; it reads no row.
/// The terminal hides the column whole (`expr::reads_row` skips quoted strings); React's
/// `readsRow` regex matches `row.` inside the quotes, keeps the column and empties every cell.
#[test]
fn a_row_dot_inside_a_quoted_text_reads_no_row() {
    run(
        "quoted",
        r"
  await settle();
  const heads = tagged(section('table'), 'th').map(textOf);
  assert.ok(heads.some((head) => /^id$/i.test(head)), `the table is drawn: ${heads}`);
  assert.ok(!heads.some((head) => /secret/i.test(head)), `a condition that reads no row hides the whole column, as the terminal does: ${heads}`);
",
    );
}

/// `params.show-row.on` is the path `params` / `show-row` / `on` in both grammars (a name may hold
/// `-`). The terminal's `reads_row` treats `-` as part of the word and hides the column whole;
/// React's `readsRow` takes `-` as a boundary, sees `row.on`, and keeps the column.
#[test]
fn a_hyphenated_name_ending_in_row_reads_no_row() {
    run(
        "hyphen",
        r"
  await settle();
  const heads = tagged(section('table'), 'th').map(textOf);
  assert.ok(heads.some((head) => /^id$/i.test(head)), `the table is drawn: ${heads}`);
  assert.ok(!heads.some((head) => /code/i.test(head)), `a condition that reads no row hides the whole column, as the terminal does: ${heads}`);
",
    );
}

/// The React half of the mixed case: `row.has_note or params.reveal` is judged per row, so the
/// first row's note is drawn and the second row's cell is empty. The terminal must draw the same.
#[test]
fn a_condition_mixing_row_and_params_is_judged_per_row() {
    run(
        "mixed",
        r"
  await settle();
  const table = section('table');
  const rows = tagged(tagged(table, 'tbody')[0], 'tr');
  assert.equal(rows.length, 2, textOf(table));
  const notes = rows.map((row) => tagged(row, 'td').map(textOf)[1]);
  assert.deepEqual(notes, ['fragile', ''], `the note cell per row: ${textOf(table)}`);
",
    );
}
