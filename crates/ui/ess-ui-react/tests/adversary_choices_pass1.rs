//! Adversary pass 1 for beyond10x/ess#328: the React runtime choice "copies the rules" of
//! `ess_ui::Choice::row_option`, which the terminal renderer and `ess ui test` call. Each case
//! runs one row through both and compares what is offered: the value sent and the text shown
//! (the terminal shows a label through `ess_ui_tui::expr::display`, which draws `null` as the
//! empty string).
//!
//! The runtime is compiled to `CommonJS` with `tsc` and run under `node`, with `core`, `data` and
//! `expr` stubbed, as `tests/choice_value.rs` does.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_yaml::Value;

const DOCUMENT: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.new
  sections: [{name: all, pages: [releases.new]}]
pages:
  releases.new:
    kind: detail_page
    title: New release
    sections:
      - {name: pick, component: choice, reads: {placeholder: release.Repositories, fixture: repositories.yaml}}
";

const FIXTURE: &str = r"
view: release.Repositories
rows:
  - {repository_id: repo-1, location: example/one}
";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-choices-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is created");
    std::fs::write(path, text).expect("the file is written");
}

fn run(program: &str, args: &[&str], dir: &Path) -> (bool, String) {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|error| panic!("{program} is on PATH: {error}"));
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

const JSX_STUB: &str = r#""use strict";
function jsx(type, props) { return { type, props: props || {} }; }
module.exports = { jsx, jsxs: jsx, Fragment: (props) => props.children };
"#;

const CORE_STUB: &str = r#""use strict";
exports.useScope = () => globalThis.__scope;
exports.useUiPath = (path) => path;
"#;

const DATA_STUB: &str = r#""use strict";
exports.useRead = () => ({ rows: globalThis.__rows });
"#;

const EXPR_STUB: &str = r#""use strict";
exports.evaluate = () => undefined;
"#;

/// Prints the options the runtime choice offers for `rows` and `props`, as JSON
/// `[[value, label], …]`, and what picking the first option writes.
const PROBE: &str = r#""use strict";
const { ChoiceView } = require("./out/src/runtime/composites/choice");
const c = JSON.parse(process.argv[2]);
const picked = [];
globalThis.__scope = { values: {}, set: (path, value) => picked.push(value) };
globalThis.__rows = c.rows;
function find(node, tag, out) {
  if (Array.isArray(node)) { node.forEach((n) => find(n, tag, out)); return out; }
  if (node && typeof node === "object" && node.props) {
    if (node.type === tag) out.push(node);
    find(node.props.children, tag, out);
  }
  return out;
}
const tree = ChoiceView({ reads: { view: "release.Repositories" }, binds: "draft.x", ...c.props });
const options = find(tree, "option", []).slice(1).map((o) => [o.props.value, o.props.children]);
const [select] = find(tree, "select", []);
if (options.length > 0) select.props.onChange({ target: { value: options[0][0] } });
process.stdout.write(JSON.stringify({ options, picked }));
"#;

/// Numbers the runtime projects, so tests running in parallel do not share (and remove) one.
static RUNTIMES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The compiled runtime project.
fn runtime() -> PathBuf {
    let serial = RUNTIMES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = scratch(&format!("runtime-{}-{serial}", std::process::id()));
    write(&dir.join("repositories.yaml"), FIXTURE);
    let document = ess_ui::load_str(DOCUMENT).expect("the document loads");
    let project = dir.join("app");
    ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    write(
        &project.join("node_modules/react/package.json"),
        r#"{"name":"react","main":"index.js","type":"commonjs"}"#,
    );
    write(
        &project.join("node_modules/react/index.js"),
        "\"use strict\";\n",
    );
    write(&project.join("node_modules/react/jsx-runtime.js"), JSX_STUB);
    write(
        &project.join("tsconfig.choice.json"),
        r#"{
  "extends": "./tsconfig.offline.json",
  "compilerOptions": {
    "noEmit": false, "outDir": "out", "rootDir": ".", "module": "commonjs",
    "moduleResolution": "node10", "ignoreDeprecations": "6.0", "noEmitOnError": false
  },
  "include": [],
  "files": ["src/runtime/composites/choice.tsx"]
}"#,
    );
    let (_, compiled) = run("tsc", &["-p", "tsconfig.choice.json"], &project);
    let out = project.join("out/src/runtime");
    assert!(
        out.join("composites/choice.js").exists(),
        "tsc emitted the choice: {compiled}"
    );
    write(&project.join("out/package.json"), r#"{"type":"commonjs"}"#);
    write(&out.join("core.js"), CORE_STUB);
    write(&out.join("data.js"), DATA_STUB);
    write(&out.join("expr.js"), EXPR_STUB);
    write(&project.join("probe.cjs"), PROBE);
    project
}

/// What the React runtime offers: `[(value, label)]`, and the value picking the first sends.
fn react(
    project: &Path,
    props: &serde_json::Value,
    rows: &serde_json::Value,
) -> (Vec<(serde_json::Value, String)>, Vec<serde_json::Value>) {
    let case = serde_json::json!({"props": props, "rows": rows});
    let (ok, output) = run("node", &["probe.cjs", &case.to_string()], project);
    assert!(ok, "{output}");
    let seen: serde_json::Value = serde_json::from_str(&output).expect("the probe prints JSON");
    let options = seen["options"]
        .as_array()
        .expect("options")
        .iter()
        .map(|pair| {
            (
                pair[0].clone(),
                pair[1].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    let picked = seen["picked"].as_array().expect("picked").clone();
    (options, picked)
}

/// The terminal's text for a label (`ess_ui_tui::expr::display` for scalars).
fn shown(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        other => panic!("not a scalar: {other:?}"),
    }
}

fn choice(yaml: &str) -> ess_ui::Choice {
    serde_yaml::from_str(yaml).unwrap_or_else(|error| panic!("{yaml}: {error}"))
}

/// A named `label` field that a row holds as `null` (an `Optional` row field without a value):
/// `row_option` takes the null as the label, so the terminal shows an empty option, while the
/// React runtime shows the text `null`. The schema note says a label "absent, or missing from a
/// row" falls back to the row's `label`, else its `name`, else the value; neither renderer shows
/// the row's `name`, and they do not show the same thing.
#[test]
fn a_null_named_label_is_shown_alike_by_both_renderers() {
    let project = runtime();
    let rows = serde_json::json!([{"repository_id": "repo-1", "nickname": null, "name": "One"}]);
    let (react_options, _) = react(
        &project,
        &serde_json::json!({"valueField": "repository_id", "labelField": "nickname"}),
        &rows,
    );
    let definition =
        choice("{reads: {view: release.Repositories}, value: repository_id, label: nickname}");
    let row: Value = serde_yaml::from_str("{repository_id: repo-1, nickname: null, name: One}")
        .expect("the row parses");
    let (value, label) = definition
        .row_option(&row, None, None)
        .expect("the row offers an option");
    let terminal = vec![(serde_json::json!(shown(&value)), shown(&label))];
    assert_eq!(
        react_options, terminal,
        "React and the terminal offer the same option for the same row"
    );
}

/// A form field's choice whose row holds the field named like the form field as `null`:
/// `row_option` sends `null`; the React runtime's `??` skips the null and sends the view's
/// identity instead.
#[test]
fn a_null_same_name_field_sends_the_same_value_in_both_renderers() {
    let project = runtime();
    let rows = serde_json::json!([{"parent_id": null, "item_id": "item-1", "label": "One"}]);
    let (_, picked) = react(
        &project,
        &serde_json::json!({"valueKey": "parent_id", "identity": "item_id"}),
        &rows,
    );
    let definition = choice("{reads: {view: shop.Items}}");
    let row: Value = serde_yaml::from_str("{parent_id: null, item_id: item-1, label: One}")
        .expect("the row parses");
    let (value, _) = definition
        .row_option(&row, Some("parent_id"), Some("item_id"))
        .expect("the row offers an option");
    let sent = serde_json::to_value(&value).expect("the value is JSON");
    assert_eq!(
        picked,
        vec![sent],
        "React and the terminal send the same value for the same row"
    );
}
