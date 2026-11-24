//! Confirming an action runs that action, once, whether its confirm is inline or an overlay of
//! the page, and whether or not the confirm declares `does`.
//!
//! Runtime cases compile the generated runtime to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r"
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: list
        component: collection
        reads: {view: things.All}
        columns: [title]
        row_actions:
          - {name: remove, does: things.Remove, bind: {thing: row.id}, confirm: {title: Remove thing}}
          - {name: drop, does: things.Drop, bind: {thing: row.id}, confirm: ask}
          - {name: tag, does: things.Tag, bind: {things: [row.id, row.title]}}
    overlays:
      ask: {kind: dialog, component: confirm, title: Drop thing}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-confirm")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    compile(
        &out,
        &[
            "runtime/actions.tsx",
            "runtime/overlays.tsx",
            "runtime/composites/confirm.tsx",
        ],
    );
    out
}

/// Renders `action` on row `th-2` (inside an overlay host holding `ask`, a confirm without
/// `does`), clicks it, clicks the confirm's primary button and prints what was sent.
const SCRIPT: &str = r#"
const { ActionControl } = out("runtime/actions");
const { OverlayHost } = out("runtime/overlays");
const { ConfirmView } = out("runtime/composites/confirm");
const { ScopeLayer } = out("runtime/core");
const { setDataAdapter } = out("runtime/data");
async function confirmed(action) {
  const sent = [];
  setDataAdapter({
    read: async () => ({ rows: [] }),
    send: async (command, input) => { sent.push([command, input]); return { accepted: true }; },
    loadState: async () => undefined,
    saveState: async () => undefined,
  });
  const overlays = { ask: () => jsx(ConfirmView, { "data-ui-path": "pages/things.list/overlays/ask" }) };
  const root = React.__root(
    jsx(OverlayHost, { overlays, children: jsx(ScopeLayer, { values: { row: { id: "th-2" } }, children: jsx(ActionControl, { action }) }) }),
  );
  root.settle();
  click(root.find((n) => n.tag === "button" && n.props["data-ui-path"] === action["data-ui-path"])[0]);
  root.settle();
  const primary = root.find((n) => n.tag === "button" && /ui-emphasis-(danger|primary)/.test(n.props.className || ""));
  assert.strictEqual(primary.length, 1, "the confirm is open");
  click(primary[0]);
  for (let i = 0; i < 10; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
  root.settle();
  const still = root.find((n) => n.tag === "button" && /ui-emphasis-(danger|primary)/.test(n.props.className || ""));
  assert.strictEqual(still.length, 0, "the confirm closes");
  return sent;
}
(async () => {
  const path = "pages/things.list/sections/list/row_actions/remove";
  // Inline, its overlay's `does` the action's own (as `confirm: {title}` expands): sent once,
  // with the action's bind.
  assert.deepStrictEqual(
    await confirmed({
      "data-ui-path": path, name: "remove", does: "things.Remove", bind: { thing: "row.id" },
      confirm: jsx(ConfirmView, { "data-ui-path": `${path}/confirm/overlay`, does: "things.Remove" }),
    }),
    [["things.Remove", { thing: "th-2" }]],
  );
  // Inline, its overlay without `does`: the action still runs.
  assert.deepStrictEqual(
    await confirmed({
      "data-ui-path": path, name: "remove", does: "things.Remove", bind: { thing: "row.id" },
      confirm: jsx(ConfirmView, { "data-ui-path": `${path}/confirm/overlay` }),
    }),
    [["things.Remove", { thing: "th-2" }]],
  );
  // `confirm: ask`, a page overlay without `does`: the action runs.
  assert.deepStrictEqual(
    await confirmed({ "data-ui-path": path, name: "drop", does: "things.Drop", bind: { thing: "row.id" }, confirmOpens: "ask" }),
    [["things.Drop", { thing: "th-2" }]],
  );
  // A `bind` value written as a list sends one list of the values its expressions read
  // (beyond10x/ess#723).
  assert.deepStrictEqual(
    await confirmed({
      "data-ui-path": path, name: "remove", does: "things.Remove", bind: { things: ["row.id", "'x'"] },
      confirm: jsx(ConfirmView, { "data-ui-path": `${path}/confirm/overlay` }),
    }),
    [["things.Remove", { things: ["th-2", "x"] }]],
  );
})().catch((error) => { console.error(error); process.exit(1); });
"#;

#[test]
fn confirming_runs_the_action_once_whether_or_not_the_confirm_declares_does() {
    let project = project("runs");
    node(&project, "confirm", SCRIPT);
    // The generated page hands the listed `bind` value over as a list.
    let page = std::fs::read_dir(project.join("src/pages"))
        .expect("the pages are generated")
        .map(|entry| std::fs::read_to_string(entry.expect("an entry").path()).expect("a page"))
        .collect::<String>();
    assert!(
        page.contains(r#"bind: { things: ["row.id", "row.title"] }"#),
        "{page}"
    );
}

/// An action names its icon and its emphasis (beyond10x/ess#723): the spec carries the icon, its
/// glyph and the tone; an `as: icon` action draws the glyph alone with its label as accessible
/// name and title, any other draws the glyph before the label, and `tone` picks the emphasis.
#[test]
fn an_action_draws_its_icon_glyph_and_its_tone() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-confirm")
        .join("action-look");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let text = DOCUMENT.replace(
        "          - {name: tag, does: things.Tag, bind: {things: [row.id, row.title]}}\n",
        "          - {name: tag, does: things.Tag, bind: {things: [row.id, row.title]}}\n          \
         - {name: archive, does: things.Archive, as: icon, icon: archive, tone: danger, label: Archive}\n",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(out.join("src/pages/ThingsList.tsx")).expect("the page");
    for written in [r#"icon: "archive""#, r#"glyph: "▤""#, r#"tone: "danger""#] {
        assert!(page.contains(written), "{written}: {page}");
    }
    let css = std::fs::read_to_string(out.join("src/runtime/styles.css")).expect("the styles");
    assert!(
        !css.contains("content: \"●\""),
        "every icon draws one glyph: {css}"
    );
    compile(&out, &["runtime/actions.tsx"]);
    node(
        &out,
        "action-look",
        r#"
const { ActionControl } = out("runtime/actions");
function drawn(action) {
  const root = React.__root(jsx(ActionControl, { action }));
  root.settle();
  return root.find((n) => n.tag === "button")[0];
}
function texts(node) {
  if (typeof node === "string") return [node];
  return (node.children || []).flatMap(texts);
}
const icon = drawn({ "data-ui-path": "a", name: "archive", does: "things.Archive", as: "icon", icon: "archive", glyph: "▤", tone: "danger", label: "Archive" });
assert.match(icon.props.className, /ui-emphasis-danger/);
assert.strictEqual(icon.props["aria-label"], "Archive");
assert.strictEqual(icon.props.title, "Archive");
assert.deepStrictEqual(texts(icon), ["▤"]);
const labelled = drawn({ "data-ui-path": "b", name: "copy", does: "things.Copy", icon: "copy", glyph: "⧉", tone: "primary", label: "Copy" });
assert.match(labelled.props.className, /ui-emphasis-primary/);
assert.deepStrictEqual(texts(labelled), ["⧉", "Copy"]);
const plain = drawn({ "data-ui-path": "c", name: "drop", does: "things.Drop", label: "Drop" });
assert.doesNotMatch(plain.props.className, /ui-emphasis/);
assert.deepStrictEqual(texts(plain), ["Drop"]);
"#,
    );
}
