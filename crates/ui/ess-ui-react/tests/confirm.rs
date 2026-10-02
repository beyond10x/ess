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
  const primary = root.find((n) => n.tag === "button" && /ui-tone-(danger|primary)/.test(n.props.className || ""));
  assert.strictEqual(primary.length, 1, "the confirm is open");
  click(primary[0]);
  for (let i = 0; i < 10; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
  root.settle();
  const still = root.find((n) => n.tag === "button" && /ui-tone-(danger|primary)/.test(n.props.className || ""));
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
})().catch((error) => { console.error(error); process.exit(1); });
"#;

#[test]
fn confirming_runs_the_action_once_whether_or_not_the_confirm_declares_does() {
    let project = project("runs");
    node(&project, "confirm", SCRIPT);
}
