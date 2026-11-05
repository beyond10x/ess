//! Adversary pass 1 over the UI work of the 0.87.0 wave: cases that fail on the integration head.
//! Each test names the defect it shows.
//!
//! - `defect_an_on_change_form_saves_a_change_made_in_a_part`: a `save: on_change` form draws no
//!   Save button, and its `parts` are rendered outside the layer that saves on change, so a value
//!   chosen in a part is never sent (story:choices-filter-bars-and-forms-render-what-they-declare).
//! - `defect_a_read_given_the_search_state_parent_draws_the_search_box`: `read_takes` matches the
//!   search's own path or a field inside it, never a read given the record that holds it, so the
//!   box is dropped while the read takes the search (same story, A15).
//! - `defect_a_placeholder_fixture_under_a_dot_slash_dir_names_the_same_file`: the schema says a
//!   placeholder `fixture:` that starts with `fixtures.dir` names the same file; under
//!   `dir: ./data`, `data/quarters.yaml` is joined as `data/data/quarters.yaml`
//!   (story:a-value-renders-in-the-form-its-field-declares, A41).

mod support;

use std::path::{Path, PathBuf};

fn document(kind: &str, state: &str, sections: &str) -> String {
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
    kind: {kind}
    title: Items
    state:
{state}
    sections:
{sections}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w087-ui-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { ShelfItemsPage } = out('pages/ShelfItems');
const rows = [{ id: 'i-1', title: 'Alpha', stage: 'open' }];
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
const tagged = (tag) => root.find((node) => node.tag === tag);
";

fn run(test: &str, yaml: &str, body: &str) {
    let out = scratch(test);
    let document =
        ess_ui::load_str(yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    support::compile(&out, &["pages/ShelfItems.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// Defect: a `save: on_change` form has no Save button, and a part's change is not saved either,
/// so it is lost.
#[test]
fn defect_an_on_change_form_saves_a_change_made_in_a_part() {
    let sections = "      - name: settings
        component: form
        does: shelf.Rename
        save: on_change
        fields: [title]
        parts:
          - {name: stage, component: choice, binds: draft.stage, options: [open, won]}";
    run(
        "on_change_part",
        &document("settings_page", "      {}", sections),
        r"
  await settle();
  const buttons = tagged('button');
  assert.ok(!buttons.some((button) => button.props.type === 'submit'), 'the on_change form draws no submit');
  const [select] = tagged('select');
  assert.ok(select, `the part's choice is drawn: ${root.text()}`);
  select.props.onChange({ target: { value: 'won' } });
  for (let i = 0; i < 10; i += 1) await tick();
  await settle();
  assert.deepStrictEqual(
    sent.map(([command, input]) => [command, input.stage]),
    [['shelf.Rename', 'won']],
    'a value chosen in a part of an on_change form is saved, as a field change is; with no Save button it is otherwise lost',
  );
",
    );
}

/// Defect: a read given the record that holds the search state takes the search, and the bar
/// still draws no search box.
#[test]
fn defect_a_read_given_the_search_state_parent_draws_the_search_box() {
    let state = "      filters: {type: {record: {q: string, stage: string}}, class: page_state, store: memory}";
    let sections = "      - name: bar
        component: filter_bar
        binds: [state.filters.q, state.filters.stage]
        search: {binds: state.filters.q}
      - name: list
        component: collection
        reads: {view: shelf.Items, params: {filters: state.filters}}
        columns: [title]";
    run(
        "search_parent",
        &document("detail_page", state, sections),
        r"
  await settle();
  const boxes = root.find((node) => node.tag === 'input' && node.props.type === 'search');
  assert.equal(boxes.length, 1, `the read is given state.filters, which holds the search state.filters.q, so the box is drawn: ${root.text()}`);
",
    );
}

const PLAN: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: ./data, views: {}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: plan
  sections: [{name: all, pages: [plan]}]
pages:
  plan:
    kind: list_page
    title: Plan
    sections:
      - name: quarters
        component: collection
        reads: {placeholder: t.Quarters, fixture: data/quarters.yaml}
        columns: [quarter]
";

/// Defect: under `dir: ./data`, a placeholder `fixture: data/quarters.yaml` starts with the
/// declared directory and names `data/quarters.yaml`, yet it is looked up as
/// `data/data/quarters.yaml` (`Path::strip_prefix` compares `./data` and `data` component-wise).
#[test]
fn defect_a_placeholder_fixture_under_a_dot_slash_dir_names_the_same_file() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w087-ui-pass1")
        .join("dot-slash-dir");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(dir.join("data")).expect("the fixture dir is created");
    std::fs::write(
        dir.join("data/quarters.yaml"),
        "views:\n  t.Quarters:\n    rows:\n      - {id: q-1, quarter: from-the-fixture-dir}\n",
    )
    .expect("the fixture is written");
    let document = ess_ui::load_str(PLAN).unwrap_or_else(|error| panic!("{error}"));
    let files = ess_ui_react::render(&document, &dir).unwrap_or_else(|error| {
        panic!("`data/quarters.yaml` under `dir: ./data` names data/quarters.yaml: {error}")
    });
    let fixtures = files
        .files
        .iter()
        .find(|(path, _)| path.as_str() == "src/fixtures.ts")
        .map(|(_, text)| text.clone())
        .expect("the project has src/fixtures.ts");
    assert!(fixtures.contains("from-the-fixture-dir"), "{fixtures}");
}
