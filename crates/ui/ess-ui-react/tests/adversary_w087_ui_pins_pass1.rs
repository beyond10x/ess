//! Adversary pass 1 over the UI work of the 0.87.0 wave: behaviour checked and found to hold,
//! pinned so a later change cannot lose it (story:a-value-renders-in-the-form-its-field-declares).
//!
//! `safeUrl` is the only guard between a row value and an `href` or a player's `src`. The cases
//! below are the ways a script or data URL is commonly smuggled past a scheme check: mixed case,
//! leading spaces and control characters, a tab or newline inside the scheme, and other schemes
//! that run or embed content. Each must render as text, never as an anchor or a player.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: calls
  sections: [{name: all, pages: [calls]}]
pages:
  calls:
    kind: list_page
    title: Calls
    sections:
      - name: calls
        component: collection
        reads: t.Calls
        columns:
          - {field: recording, as: audio}
          - {field: site, as: link}
";

fn project(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w087-ui-pins-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    compile(&project, &["runtime/fields.tsx"]);
    project
}

const RENDER: &str = r#"
const { FieldValue } = out("runtime/fields");
const shown = (field, row) => React.__root(jsx(FieldValue, { field, row })).settle();
const column = (name, extra) => ({ "data-ui-path": `pages/calls/sections/calls/columns/${name}`, field: name, name, ...extra });
const link = column("site", { as: "link" });
const audio = column("recording", { as: "audio" });
const anchors = (value) => shown(link, { site: value }).find((n) => n.tag === "a");
const players = (value) => shown(audio, { recording: value }).find((n) => n.tag === "audio");
"#;

#[test]
fn pin_no_script_or_data_url_becomes_an_anchor_or_a_player() {
    let project = project("refused");
    node(
        &project,
        "refused",
        &format!(
            r#"{RENDER}
const refused = [
  "JavaScript:alert(1)",
  "JAVASCRIPT:alert(1)",
  "  javascript:alert(1)",
  "\u0000javascript:alert(1)",
  "\u001fjavascript:alert(1)",
  "java\tscript:alert(1)",
  "java\nscript:alert(1)",
  "javascript\t:alert(1)",
  "\njavascript:alert(1)",
  "javascript:alert(1)\n",
  "vbscript:msgbox(1)",
  "data:text/html,<script>alert(1)</script>",
  "DATA:text/html;base64,PHNjcmlwdD4=",
  "blob:https://media.test/1",
  "file:///etc/passwd",
  "ftp://media.test/a.mp3",
  "",
  "   ",
];
for (const value of refused) {{
  assert.strictEqual(anchors(value).length, 0, `no anchor for ${{JSON.stringify(value)}}`);
  assert.strictEqual(players(value).length, 0, `no player for ${{JSON.stringify(value)}}`);
}}
// A value that is no text is no address either.
for (const value of [42, true, {{ href: "https://shop.test" }}, ["https://shop.test"]]) {{
  assert.strictEqual(anchors(value).length, 0, `no anchor for ${{JSON.stringify(value)}}`);
  assert.strictEqual(players(value).length, 0, `no player for ${{JSON.stringify(value)}}`);
}}
"#
        ),
    );
}

#[test]
fn pin_http_https_mailto_and_relative_addresses_are_kept_as_written() {
    let project = project("kept");
    node(
        &project,
        "kept",
        &format!(
            r##"{RENDER}
const kept = [
  "https://shop.test/items/7",
  "HTTP://shop.test/items/7",
  "mailto:desk@shop.test",
  "/items/7",
  "items/7",
  "?page=2",
  "#top",
  // Scheme-relative: the browser loads it with the page's own scheme, http or https.
  "//media.test/a.mp3",
];
for (const value of kept) {{
  const found = anchors(value);
  assert.strictEqual(found.length, 1, `an anchor for ${{JSON.stringify(value)}}`);
  assert.strictEqual(found[0].props.href, value, "the address is passed as written");
}}
// A text that only looks like a scheme after an invisible character stays a relative address,
// as the browser reads it, so it cannot run.
const hidden = anchors("java\u0000script:alert(1)");
if (hidden.length === 1) {{
  assert.strictEqual(new URL(hidden[0].props.href, "https://shop.test/").protocol, "https:");
}}
"##
        ),
    );
}
