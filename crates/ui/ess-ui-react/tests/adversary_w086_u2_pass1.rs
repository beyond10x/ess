//! Adversary pass 1 on story:a-toggle-shows-the-row-state-it-flips, React side: the same
//! `not (row.enabled)` toggle the terminal case drives. The library names the state
//! `(row.enabled)`; this records what the React runtime shows for it.

mod support;

use std::path::{Path, PathBuf};

fn document(row_actions: &str) -> String {
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
        columns: [title]
{row_actions}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { FlagsListPage } = out('pages/FlagsList');
const rows = [
  { id: 'f-1', title: 'Alpha', enabled: true },
  { id: 'f-2', title: 'Beta', enabled: false },
];
setDataAdapter({
  read: async () => ({ rows: rows.map((row) => ({ ...row })), total: rows.length }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(FlagsListPage, {}));
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

#[test]
fn a_parenthesised_not_bind_is_checked_from_the_row() {
    let out = scratch("parenthesised");
    let yaml = document(
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not (row.enabled)}}",
    );
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("{error}"));
    support::compile(&out, &["pages/FlagsList.tsx"]);
    support::node(
        &out,
        "parenthesised",
        &format!(
            "{HARNESS}\n(async () => {{\n  await settle();\n  assert.deepStrictEqual(toggles().map((input) => input.props.checked), [true, false], 'React reads (row.enabled)');\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}
