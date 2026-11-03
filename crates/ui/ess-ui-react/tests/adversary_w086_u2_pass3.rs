//! Adversary pass 3 on story:a-toggle-shows-the-row-state-it-flips, React side, against fix
//! round 2. The library writes a derived state in the filter grammar and the terminal now reads
//! it with that grammar; the React runtime still reads it with `evaluate` (`expr.ts.tmpl`), the
//! display grammar. Each case here has a terminal twin in `ess-ui-tui`'s pass-3 file on the same
//! rows, so the two renderers can be compared.

mod support;

use std::path::{Path, PathBuf};

fn document(action: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: posts
model: posts.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: posts.list
  sections: [{{name: all, pages: [posts.list]}}]
pages:
  posts.list:
    kind: detail_page
    title: Posts
    sections:
      - name: list
        component: collection
        reads: {{view: posts.All}}
        columns: [title]
        row_actions:
          - {action}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u2-pass3")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { PostsListPage } = out('pages/PostsList');
setDataAdapter({
  read: async () => ({ rows: rows.map((row) => ({ ...row })), total: rows.length }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(PostsListPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
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
const toggles = () => root
  .find((node) => node.tag === 'label' && node.props.className === 'ui-toggle')
  .map((label) => inside(label, checkbox)[0]);
";

fn run(name: &str, action: &str, rows: &str, expected: &str) {
    let out = scratch(name);
    let yaml = document(action);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("{error}"));
    support::compile(&out, &["pages/PostsList.tsx"]);
    support::node(
        &out,
        name,
        &format!(
            "const rows = {rows};\n{HARNESS}\n(async () => {{\n  await settle();\n  assert.deepStrictEqual(toggles().map((input) => input.props.checked), {expected}, {action:?});\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// `negated` parses `not (row.level == -1)` with the filter grammar, which reads `-1` as a
/// number, and hands `row.level == -1` to the runtime. The display tokenizer has no negative
/// numbers, so `evaluate` cannot parse the state, returns the text, and the text is truthy: every
/// row is drawn on. The terminal (pass-3 twin) draws on, off.
#[test]
fn a_flipped_comparison_with_a_negative_number_is_checked_from_the_row() {
    run(
        "negative",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not (row.level == -1)"}}"#,
        "[{ id: 'p-1', title: 'Alpha', level: -1 }, { id: 'p-2', title: 'Beta', level: 3 }]",
        "[true, false]",
    );
}

/// Twin of the terminal's red case: `in` over a row list is a display expression React reads.
#[test]
fn a_checked_membership_in_a_row_list_is_checked_from_the_row() {
    run(
        "member",
        "{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id}, checked: row.id in row.owners}",
        "[{ id: 'p-1', title: 'Alpha', owners: ['p-1'] }, { id: 'p-2', title: 'Beta', owners: ['x'] }]",
        "[true, false]",
    );
}

/// Twin of the terminal's red case: the text `false` is truthy to the runtime.
#[test]
fn a_flipped_text_field_is_checked_as_javascript_reads_it() {
    run(
        "text",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not row.enabled"}}"#,
        "[{ id: 'p-1', title: 'Alpha', enabled: 'false' }, { id: 'p-2', title: 'Beta', enabled: '' }]",
        "[true, false]",
    );
}

/// Missing fields and nulls compare as empty text on both sides.
#[test]
fn a_flipped_null_comparison_over_a_missing_field_is_checked_from_the_row() {
    run(
        "null",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not (row.gone == null and row.kind in [null, 'b c'])"}}"#,
        "[{ id: 'p-1', title: 'Alpha', kind: null }, { id: 'p-2', title: 'Beta', gone: 1, kind: 'b c' }]",
        "[true, false]",
    );
}
