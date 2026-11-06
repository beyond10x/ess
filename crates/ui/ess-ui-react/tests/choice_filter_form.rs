//! Choices, filter bars and forms render what the document declares
//! (story:choices-filter-bars-and-forms-render-what-they-declare, beyond10x/ess#723): a dropdown's
//! empty option reads as its `placeholder` or a neutral word, filter controls are labelled with
//! the words the document gives them, `multiple` with `style: dropdown` is a multi-select, a
//! filter bar draws its search box only when a read on the page takes the search, a tree choice
//! sends the chosen value, a form that saves on change has no Save button, form buttons follow the
//! declaration (submit, then the actions in order), a reset button shows its `label`, and `help`
//! is drawn on a form group, a form field and a column header.
//!
//! Each case generates one page, compiles it with `tsc` and runs it under `node` against the
//! stand-in `react` of `support`, with a data adapter that answers every read with `ROWS` and
//! records every command sent.

mod support;

use std::path::{Path, PathBuf};

/// A document with one page, `shelf.items`, whose kind, state and sections are written in.
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
        .join("ess-ui-react-choice-filter-form")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// Shared by every case: the rows every read answers, an adapter recording what is sent, the page
/// and helpers to find elements and their text.
const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { ShelfItemsPage } = out('pages/ShelfItems');
const rows = [
  { id: 'i-1', title: 'Alpha', stage: 'open' },
  { id: 'i-2', title: 'Beta', stage: 'won' },
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
const tagged = (tag) => root.find((node) => node.tag === tag);
const classed = (name) => root.find((node) => new RegExp(`\\b${name}\\b`).test(node.props.className || ''));
const selects = () => tagged('select');
// The text of each option of a select, in order.
const optionTexts = (select) => inside(select, (node) => node.tag === 'option').map(textOf);
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

const FILTER_STATE: &str = "      stage: {type: string, class: page_state, store: memory}
      owner_id: {type: string, class: page_state, store: memory}
      tags: {type: {list: string}, class: page_state, store: memory}";

/// A filter bar over `stage`, `owner_id` and `tags`, followed by `extra` keys of the bar.
fn filter_bar(choices: &str, extra: &str) -> String {
    format!(
        "      - name: filters
        component: filter_bar
        binds: [state.stage, state.owner_id, state.tags]
        choices:
{choices}
{extra}
      - name: list
        component: collection
        reads: {{view: shelf.Items, params: {{stage: state.stage, owner_id: state.owner_id, tags: state.tags}}}}
        columns: [title]"
    )
}

#[test]
fn an_empty_dropdown_option_shows_the_placeholder_or_a_neutral_word() {
    let sections = filter_bar(
        "          - {name: stage, component: choice, options: [open, won], placeholder: Any stage}
          - {name: owner_id, component: choice, options: [ann, bob]}",
        "",
    );
    run(
        "empty_option",
        &document("detail_page", FILTER_STATE, &sections),
        r"
  await settle();
  const [stage, owner] = selects();
  assert.ok(stage && owner, `two dropdowns: ${root.text()}`);
  assert.deepStrictEqual(optionTexts(stage), ['Any stage', 'open', 'won'], 'the empty option is the placeholder');
  const [empty] = optionTexts(owner);
  assert.notStrictEqual(empty, 'owner_id', 'the empty option is not the state key');
  assert.notStrictEqual(empty, 'owner id', 'the empty option is not the state key');
  assert.strictEqual(empty, 'Select…', 'without a placeholder the empty option is a neutral word');
",
    );
}

#[test]
fn filter_captions_use_the_declared_label() {
    let sections = filter_bar(
        "          - {name: stage, component: choice, options: [open, won], placeholder: Any stage}",
        "        inputs:
          - {field: owner_id, label: Owner, as: choice, binds: state.owner_id, choice: {component: choice, options: [ann, bob]}}",
    );
    run(
        "captions",
        &document("detail_page", FILTER_STATE, &sections),
        r"
  await settle();
  const labels = selects().map((select) => select.props['aria-label']);
  assert.deepStrictEqual(labels, ['Any stage', 'Owner'], 'each filter control is named by its declared words');
  for (const label of labels) {
    assert.ok(!/_|^stage$/.test(label), `no caption is a raw binding name: ${label}`);
  }
",
    );
}

#[test]
fn the_window_select_is_labelled() {
    let sections = "      - name: list
        component: collection
        reads: {view: shelf.Items, params: {window: state.window}}
        columns: [title]";
    run(
        "window",
        &document("report_page", "      {}", sections),
        r"
  await settle();
  const windows = classed('ui-window');
  assert.equal(windows.length, 1, `the report page draws its window: ${root.text()}`);
  const [select] = inside(windows[0], (node) => node.tag === 'select');
  assert.ok(select, 'the window holds a select');
  const label = select.props['aria-label'];
  assert.ok(typeof label === 'string' && label.trim().length > 0, `the window select has a label: ${label}`);
  assert.ok(!/_/.test(label), `the label reads as words: ${label}`);
",
    );
}

#[test]
fn a_multiple_dropdown_is_a_multi_select() {
    let sections = filter_bar(
        "          - {name: tags, component: choice, multiple: true, style: dropdown, options: [red, green, blue]}",
        "",
    );
    run(
        "multiple",
        &document("detail_page", FILTER_STATE, &sections),
        r"
  await settle();
  const [select] = selects();
  assert.ok(select, `the dropdown is a select: ${root.text()}`);
  assert.strictEqual(select.props.multiple, true, 'a multiple dropdown is a multi-select');
  assert.deepStrictEqual(optionTexts(select), ['red', 'green', 'blue'], 'a multi-select offers no empty option');
  select.props.onChange({ target: { value: 'red', selectedOptions: [{ value: 'red' }, { value: 'blue' }] } });
  await settle();
  const [again] = selects();
  assert.deepStrictEqual(again.props.value, ['red', 'blue'], 'both chosen values are held');
",
    );
}

#[test]
fn a_list_without_a_search_param_draws_no_search_box() {
    let search_boxes = r"
  await settle();
  const boxes = root.find((node) => node.tag === 'input' && node.props.type === 'search');
";
    let unread = "      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns: [title]";
    run(
        "search_unread",
        &document("list_page", "      {}", unread),
        &format!(
            "{search_boxes}  assert.equal(boxes.length, 0, `no read takes the search, so no search box: ${{root.text()}}`);\n"
        ),
    );
    let read = "      - name: list
        component: collection
        reads: {view: shelf.Items, params: {q: state.search}}
        columns: [title]";
    run(
        "search_read",
        &document("list_page", "      {}", read),
        &format!(
            "{search_boxes}  assert.equal(boxes.length, 1, `a read takes the search, so the box is drawn: ${{root.text()}}`);\n"
        ),
    );
}

/// A read given a sibling of the search field, or a record whose name only starts with the name
/// of the record holding the search, takes neither the search nor a record holding it, so the bar
/// draws no search box (story:a-read-given-the-search-state-record-draws-the-search-box).
#[test]
fn a_read_given_neither_the_search_nor_a_record_holding_it_draws_no_search_box() {
    let state = "      filters: {type: {record: {q: string, stage: string}}, class: page_state, store: memory}
      filters_saved: {type: {record: {q: string}}, class: page_state, store: memory}";
    let sections = "      - name: bar
        component: filter_bar
        binds: [state.filters.q, state.filters.stage]
        search: {binds: state.filters.q}
      - name: list
        component: collection
        reads: {view: shelf.Items, params: {stage: state.filters.stage, saved: state.filters_saved}}
        columns: [title]";
    run(
        "search_sibling",
        &document("detail_page", state, sections),
        r"
  await settle();
  const boxes = root.find((node) => node.tag === 'input' && node.props.type === 'search');
  assert.equal(boxes.length, 0, `the read is given state.filters.stage and state.filters_saved, neither of which holds state.filters.q: ${root.text()}`);
",
    );
}

#[test]
fn a_tree_choice_sends_the_chosen_value() {
    let sections = "      - name: form
        component: form
        does: shelf.Move
        fields:
          - {field: title}
          - field: folder_id
            label: Folder
            as: choice
            choice: {component: choice, style: tree, reads: {view: shelf.Items}}";
    run(
        "tree",
        &document("form_page", "      {}", sections),
        r"
  await settle();
  const [select] = selects();
  assert.ok(select, `the tree choice is drawn: ${root.text()}`);
  assert.deepStrictEqual(
    inside(select, (node) => node.tag === 'option').slice(1).map((option) => option.props.value),
    ['i-1', 'i-2'],
    'each option carries its row value',
  );
  select.props.onChange({ target: { value: 'i-2', selectedOptions: [{ value: 'i-2' }] } });
  await settle();
  const [form] = tagged('form');
  form.props.onSubmit({ preventDefault() {} });
  for (let i = 0; i < 10; i += 1) await tick();
  assert.equal(sent.length, 1, `one command is sent: ${JSON.stringify(sent)}`);
  assert.strictEqual(sent[0][1].folder_id, 'i-2', `the chosen row value is sent: ${JSON.stringify(sent)}`);
",
    );
}

#[test]
fn an_on_change_form_has_no_save_button() {
    let sections = "      - name: settings
        component: form
        does: shelf.Rename
        save: on_change
        fields: [title]";
    run(
        "on_change",
        &document("settings_page", "      {}", sections),
        r"
  await settle();
  const buttons = tagged('button');
  assert.deepStrictEqual(
    buttons.filter((button) => button.props.type === 'submit').map(textOf),
    [],
    'a form that saves on change draws no submit button',
  );
  assert.ok(!buttons.some((button) => /^Save/.test(textOf(button))), `no Save button: ${buttons.map(textOf)}`);
  const [input] = root.find((node) => node.tag === 'input');
  input.props.onChange({ target: { value: 'Gamma' } });
  for (let i = 0; i < 10; i += 1) await tick();
  await settle();
  assert.deepStrictEqual(sent, [['shelf.Rename', { title: 'Gamma' }]], 'a change saves at once');
",
    );
}

#[test]
fn form_actions_render_in_declared_order() {
    let sections = "      - name: form
        component: form
        does: shelf.Create
        submit: {label: Create}
        fields: [title]
        actions:
          - {name: archive, does: shelf.Archive, label: Archive}
          - {name: purge, does: shelf.Purge, label: Purge}";
    run(
        "action_order",
        &document("form_page", "      {}", sections),
        r"
  await settle();
  const [actions] = classed('ui-form-actions');
  assert.ok(actions, `the form has its action row: ${root.text()}`);
  const order = inside(actions, (node) => node.tag === 'button').map(textOf);
  assert.deepStrictEqual(order, ['Create', 'Archive', 'Purge'], 'submit first, then the actions in declared order');
",
    );
}

#[test]
fn a_reset_button_shows_its_label() {
    let labelled = filter_bar(
        "          - {name: stage, component: choice, options: [open, won]}",
        "        reset: {label: Clear filters}",
    );
    run(
        "reset_label",
        &document("detail_page", FILTER_STATE, &labelled),
        r"
  await settle();
  const [bar] = classed('ui-filter-bar');
  const texts = inside(bar, (node) => node.tag === 'button').map(textOf);
  assert.deepStrictEqual(texts, ['Clear filters'], 'the reset button shows its label');
",
    );
    let plain = filter_bar(
        "          - {name: stage, component: choice, options: [open, won]}",
        "        reset: true",
    );
    run(
        "reset_true",
        &document("detail_page", FILTER_STATE, &plain),
        r"
  await settle();
  const [bar] = classed('ui-filter-bar');
  assert.deepStrictEqual(inside(bar, (node) => node.tag === 'button').map(textOf), ['Reset'], '`reset: true` keeps its word');
",
    );
    let none = filter_bar(
        "          - {name: stage, component: choice, options: [open, won]}",
        "",
    );
    run(
        "reset_absent",
        &document("detail_page", FILTER_STATE, &none),
        r"
  await settle();
  const [bar] = classed('ui-filter-bar');
  assert.deepStrictEqual(inside(bar, (node) => node.tag === 'button').map(textOf), [], 'no reset unless declared');
",
    );
}

#[test]
fn help_renders_on_a_form_group_a_field_and_a_column_header() {
    let sections = "      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns:
          - {field: title, label: Title, help: {text: The name shown on the shelf}}
      - name: form
        component: form
        does: shelf.Configure
        groups:
          - name: profile
            label: Profile
            help: {text: Who sees the shelf, link: 'https://example.test/profile'}
            fields:
              - {field: owner, help: {text: The person who answers for it}}";
    run(
        "help",
        &document("detail_page", "      {}", sections),
        r"
  await settle();
  const [group] = classed('ui-form-group');
  assert.ok(group, `the group is drawn: ${root.text()}`);
  assert.ok(textOf(group).includes('Who sees the shelf'), `the group shows its help: ${textOf(group)}`);
  const links = inside(group, (node) => node.tag === 'a').map((a) => a.props.href);
  assert.ok(links.includes('https://example.test/profile'), `the group links its help: ${links}`);
  const [field] = inside(group, (node) => node.tag === 'label' && /ui-form-field/.test(node.props.className || ''));
  assert.ok(field && textOf(field).includes('The person who answers for it'), 'the field shows its help');
  const [heading] = tagged('th');
  assert.ok(heading && textOf(heading).includes('The name shown on the shelf'), `the column header shows its help: ${heading && textOf(heading)}`);
",
    );
}

/// A help link goes through `safeUrl`, as a `link` value does: a script address on a column, a
/// group, a field or the page header is shown as text, never as an anchor.
#[test]
fn a_script_help_link_renders_as_text_not_an_anchor() {
    let sections = "      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns:
          - {field: title, label: Title, help: {text: Named, link: 'javascript:alert(1)'}}
      - name: form
        component: form
        does: shelf.Configure
        groups:
          - name: profile
            label: Profile
            help: {text: Who sees the shelf, link: 'JavaScript:alert(2)'}
            fields:
              - {field: owner, help: {text: The person, link: ' javascript:alert(3)'}}";
    let yaml = document("detail_page", "      {}", sections).replace(
        "    state:\n",
        "    header: {help: {text: About items, link: 'javascript:alert(4)'}}\n    state:\n",
    );
    run(
        "help_script_link",
        &yaml,
        r"
  await settle();
  const hrefs = tagged('a').map((a) => String(a.props.href));
  assert.ok(!hrefs.some((href) => /script/i.test(href)), `no help link is an anchor to a script: ${hrefs}`);
  const text = root.text();
  for (const link of ['javascript:alert(1)', 'JavaScript:alert(2)', 'javascript:alert(3)', 'javascript:alert(4)']) {
    assert.ok(text.includes(link), `the refused help link ${link} is shown as text: ${text}`);
  }
",
    );
}

/// The same holds for a `link` primitive's `href`: a script address is text, an http one a link.
#[test]
fn a_script_link_primitive_renders_as_text_not_an_anchor() {
    let sections = "      - name: one
        component: record
        reads: {view: shelf.Items}
        fields: [title]
        children:
          - {name: bad, primitive: link, text: Bad site, href: 'javascript:alert(5)'}
          - {name: good, primitive: link, text: Good site, href: 'https://example.test/site'}";
    run(
        "link_primitive_script",
        &document("detail_page", "      {}", sections),
        r"
  await settle();
  const anchors = tagged('a');
  const hrefs = anchors.map((a) => String(a.props.href));
  assert.ok(!hrefs.some((href) => /script/i.test(href)), `no link is an anchor to a script: ${hrefs}`);
  assert.ok(anchors.some((a) => a.props.href === 'https://example.test/site' && textOf(a) === 'Good site'), `the http link is an anchor: ${hrefs}`);
  assert.ok(root.text().includes('Bad site'), `the refused link keeps its text: ${root.text()}`);
",
    );
}
