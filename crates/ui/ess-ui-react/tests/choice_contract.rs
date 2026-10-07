//! beyond10x/ess#328: the React runtime choice and `ess_ui::Choice::row_option` (which the
//! terminal renderer and `ess ui test` call) offer the same options for one shared list of cases:
//! the same rows offered, the same value sent for each, the same text shown.
//!
//! One step is renderer-specific and so not in the list: a row object with no field the rules
//! name. The terminal sends the row itself and React sends the row's `value`, else `null`, as
//! each did before named fields existed.

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

const FIXTURE: &str = "view: release.Repositories\nrows: []\n";

/// One case: `(name, choice, form field, view identity, rows)`.
type Case = (
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);

const CASES: &[Case] = &[
    (
        "explicit value and label",
        "{value: repository_id, label: location}",
        Some("selected"),
        Some("uid"),
        "[{repository_id: r1, location: one, selected: s, uid: u, id: i}]",
    ),
    (
        "explicit value, no label field",
        "{value: repository_id, label: location}",
        None,
        None,
        "[{repository_id: r1, name: Named}, {repository_id: r2}]",
    ),
    (
        "a row without the value field offers nothing",
        "{value: repository_id}",
        None,
        None,
        "[{location: nowhere, id: wrong}, {repository_id: r2}]",
    ),
    (
        "a null label field falls back",
        "{value: repository_id, label: nickname}",
        None,
        None,
        "[{repository_id: r1, nickname: null, name: One}, {repository_id: r2, nickname: null, label: null}]",
    ),
    (
        "the read's key",
        "{reads: {view: v, key: repository_id}}",
        Some("selected"),
        Some("uid"),
        "[{repository_id: r1, selected: s, uid: u, id: i}]",
    ),
    (
        "the same-named field",
        "{}",
        Some("selected"),
        Some("uid"),
        "[{selected: s, uid: u, id: i}]",
    ),
    (
        "a same-named field present as null is sent",
        "{}",
        Some("parent_id"),
        Some("item_id"),
        "[{parent_id: null, item_id: item-1, label: One}]",
    ),
    (
        "the view's identity",
        "{}",
        Some("selected"),
        Some("uid"),
        "[{uid: u, id: i}]",
    ),
    (
        "an identity present as null is sent",
        "{}",
        None,
        Some("uid"),
        "[{uid: null, id: i, name: N}]",
    ),
    (
        "id",
        "{}",
        Some("selected"),
        Some("uid"),
        "[{id: i, name: Named}]",
    ),
    (
        "a typed value keeps its type",
        "{value: repository_no, label: location}",
        None,
        None,
        "[{repository_no: 7, location: seven}, {repository_no: true, location: yes}]",
    ),
    (
        "scalar rows offer themselves",
        "{}",
        Some("selected"),
        None,
        "[a, 2]",
    ),
    (
        "scalar rows offer nothing for a named value",
        "{value: repository_id}",
        None,
        None,
        "[a, 2]",
    ),
];

fn scratch() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-react-choice-contract");
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

/// Renders the choice for `rows` and `props`, picks every option in turn and prints
/// `{labels, picked}`.
const PROBE: &str = r#""use strict";
const { ChoiceView } = require("./out/src/runtime/composites/choice");
const c = JSON.parse(process.argv[2]);
// A prop the generator does not write is absent, not null.
for (const key of Object.keys(c.props)) if (c.props[key] === null) delete c.props[key];
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
const tree = ChoiceView({ reads: { view: "v" }, binds: "draft.x", ...c.props });
const options = find(tree, "option", []).slice(1);
const [select] = find(tree, "select", []);
for (const option of options) select.props.onChange({ target: { value: option.props.value } });
process.stdout.write(JSON.stringify({ labels: options.map((o) => o.props.children), picked }));
"#;

fn runtime() -> PathBuf {
    let dir = scratch();
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
    write(
        &project.join("node_modules/react/jsx-runtime.js"),
        "\"use strict\";\nfunction jsx(type, props) { return { type, props: props || {} }; }\n\
         module.exports = { jsx, jsxs: jsx, Fragment: (props) => props.children };\n",
    );
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
    write(
        &out.join("core.js"),
        "\"use strict\";\nexports.useScope = () => globalThis.__scope;\nexports.useUiPath = (path) => path;\n",
    );
    write(
        &out.join("data.js"),
        "\"use strict\";\nexports.useRead = () => ({ rows: globalThis.__rows });\n",
    );
    write(
        &out.join("expr.js"),
        "\"use strict\";\nexports.evaluate = () => undefined;\n",
    );
    write(&project.join("probe.cjs"), PROBE);
    project
}

/// The text the terminal shows for a scalar (`ess_ui_tui`'s `display`).
fn shown(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        other => panic!("not a scalar: {other:?}"),
    }
}

#[test]
fn react_and_row_option_offer_the_same_options_for_every_case() {
    let project = runtime();
    for (name, yaml, field, identity, rows) in CASES {
        let choice: ess_ui::Choice = serde_yaml::from_str(&match *yaml {
            "{}" => "{reads: {view: v}}".to_owned(),
            with_reads if with_reads.contains("reads") => with_reads.to_owned(),
            other => format!("{{reads: {{view: v}}, {}", &other[1..]),
        })
        .unwrap_or_else(|error| panic!("{name}: {error}"));
        let rows: Vec<Value> = serde_yaml::from_str(rows).expect("the rows parse");
        let (values, labels): (Vec<serde_json::Value>, Vec<String>) = rows
            .iter()
            .filter_map(|row| choice.row_option(row, *field, *identity))
            .map(|(value, label)| (serde_json::to_value(&value).expect("JSON"), shown(&label)))
            .unzip();
        let props = serde_json::json!({
            "valueField": choice.value_field(),
            "labelField": choice.label,
            "valueKey": field,
            "identity": identity,
        });
        let case = serde_json::json!({"props": props, "rows": rows});
        let (ok, output) = run("node", &["probe.cjs", &case.to_string()], &project);
        assert!(ok, "{name}: {output}");
        let seen: serde_json::Value = serde_json::from_str(&output).expect("the probe prints JSON");
        assert_eq!(
            seen["labels"],
            serde_json::json!(labels),
            "{name}: the shown text"
        );
        assert_eq!(
            seen["picked"],
            serde_json::json!(values),
            "{name}: the values sent"
        );
    }
}
