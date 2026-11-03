//! Adversary pass 2 on story:a-toggle-shows-the-row-state-it-flips, React side, against fix
//! round 1. The generator hands the runtime the state `canonical` writes back; the runtime reads
//! it with `evaluate` (`expr.ts.tmpl`), where a bare root name such as `draft` is a path.

mod support;

use std::path::{Path, PathBuf};

fn document(bind: &str) -> String {
    format!(
        r#"
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
          - {{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {{id: row.id, on: "{bind}"}}}}
"#
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u2-pass2")
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
const rows = [
  { id: 'p-1', title: 'Alpha', enabled: true, ready: true, kind: 'a', stage: 'draft' },
  { id: 'p-2', title: 'Beta', enabled: false, ready: false, kind: 'z', stage: 'live' },
];
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

fn run(name: &str, bind: &str) {
    let out = scratch(name);
    let yaml = document(bind);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("{error}"));
    support::compile(&out, &["pages/PostsList.tsx"]);
    support::node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n  await settle();\n  assert.deepStrictEqual(toggles().map((input) => input.props.checked), [true, false], {bind:?});\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// The first row's stage is the text `draft`, so the operand `row.stage == 'draft'` is true there.
#[test]
fn a_flipped_comparison_with_a_quoted_root_word_is_checked_from_the_row() {
    run("draft", "not (row.stage == 'draft')");
}

/// The same toggle shape over `and`: React reads it (compare the terminal case of pass 2).
#[test]
fn a_flipped_conjunction_is_checked_from_the_row() {
    run("and", "not (row.enabled and row.ready)");
}
