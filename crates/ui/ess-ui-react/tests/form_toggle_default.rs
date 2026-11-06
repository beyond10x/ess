//! A form toggle never touched is sent, not left out
//! (story:a-journey-step-reaches-every-control-a-page-declares, its toggle decision): the
//! generated form sends it as its declared default, and as `false`, the unchecked box it shows,
//! when none is declared, as the terminal does. A toggle inside a group and one on a dotted field
//! count too.
//!
//! Each case generates one page, compiles it with `tsc` and runs it under `node` against the
//! stand-in `react` of `support`, with a data adapter that records every command sent.

mod support;

use std::path::{Path, PathBuf};

/// A document with one page, `shelf.items`, holding the form section written in.
fn document(section: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: shelf
model: shelf.system
placement_profile: fat
types:
  Prefs: {{record: {{digest: boolean, notify: boolean}}}}
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: shelf.items
  sections: [{{name: all, pages: [shelf.items]}}]
pages:
  shelf.items:
    kind: settings_page
    title: Items
    sections:
{section}
"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-form-toggle-default")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// Shared by every case: an adapter recording what is sent, the page, and helpers to find the
/// form and its checkboxes.
const HARNESS: &str = r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { ShelfItemsPage } = out('pages/ShelfItems');
const sent = [];
setDataAdapter({
  read: async () => ({ rows: [], total: 0 }),
  send: async (command, input) => { sent.push([command, input]); return { accepted: true }; },
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(ShelfItemsPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const checkboxes = () => root.find((node) => node.tag === 'input' && node.props.type === 'checkbox');
async function submit() {
  const [form] = root.find((node) => node.tag === 'form');
  assert.ok(form, `a form is drawn: ${root.text()}`);
  form.props.onSubmit({ preventDefault() {} });
  for (let i = 0; i < 10; i += 1) await tick();
}
";

fn run(test: &str, section: &str, body: &str) {
    let out = scratch(test);
    let yaml = document(section);
    let document =
        ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
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

#[test]
fn an_untouched_toggle_is_sent_as_false() {
    let section = "      - name: settings
        component: form
        does: shelf.Save
        fields:
          - title
          - {field: notify, as: toggle, label: Notify}
          - {field: digest, as: toggle, label: Digest}
          - {field: prefs.weekly, as: toggle, label: Weekly}
        groups:
          - name: extra
            label: Extra
            fields:
              - {field: loud, as: toggle, label: Loud}";
    run(
        "untouched",
        section,
        r"
  await settle();
  const boxes = checkboxes();
  assert.equal(boxes.length, 4, `four toggles are drawn: ${root.text()}`);
  boxes[0].props.onChange({ target: { checked: true } });
  await settle();
  await submit();
  assert.equal(sent.length, 1, `one command is sent: ${JSON.stringify(sent)}`);
  assert.deepStrictEqual(
    sent[0],
    ['shelf.Save', { notify: true, digest: false, prefs: { weekly: false }, loud: false }],
    'each untouched toggle is sent as false, the checked one as true',
  );
",
    );
}

#[test]
fn an_untouched_toggle_is_sent_as_its_declared_default() {
    let section = "      - name: settings
        component: form
        does: shelf.SavePrefs
        draft: {type: Prefs, class: draft, store: memory, default: {digest: true}}
        fields:
          - {field: digest, as: toggle, label: Digest}
          - {field: notify, as: toggle, label: Notify}";
    run(
        "declared",
        section,
        r"
  await settle();
  await submit();
  assert.equal(sent.length, 1, `one command is sent: ${JSON.stringify(sent)}`);
  assert.deepStrictEqual(
    sent[0],
    ['shelf.SavePrefs', { digest: true, notify: false }],
    'the declared default is sent, and false where none is declared',
  );
",
    );
}
