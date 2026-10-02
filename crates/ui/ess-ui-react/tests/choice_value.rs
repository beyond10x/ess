//! A form field drawn as a choice over a view whose rows carry no `id`: the generated choice
//! takes each option's value from the row's key named like the field, and shows a readable label.
//!
//! The runtime case compiles the choice composite to `CommonJS` with `tsc` and runs it under
//! `node`, with `core`, `data` and `expr` replaced by stubs in the scratch project.

use std::path::{Path, PathBuf};
use std::process::Command;

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
    kind: form_page
    title: New release
    sections:
      - name: form
        component: form
        does: release.Cut
        fields:
          - field: repository_id
            as: choice
            choice: {component: choice, reads: {placeholder: release.Repositories, fixture: repositories.yaml}}
";

const FIXTURE: &str = r"
view: release.Repositories
rows:
  - {repository_id: repo-1, location: example/one}
  - {repository_id: repo-2, location: example/two}
";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-choice")
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

/// Renders the choice with the props the generated page passes it and picks the first option.
const SCRIPT: &str = r#""use strict";
const assert = require("assert");
const { ChoiceView } = require("./out/src/runtime/composites/choice");
const props = JSON.parse(process.argv[2]);
const picked = [];
globalThis.__scope = { values: {}, set: (path, value) => picked.push([path, value]) };
globalThis.__rows = [
  { repository_id: "repo-1", location: "example/one" },
  { repository_id: "repo-2", location: "example/two" },
];
function find(node, tag, out) {
  if (Array.isArray(node)) { node.forEach((n) => find(n, tag, out)); return out; }
  if (node && typeof node === "object" && node.props) {
    if (node.type === tag) out.push(node);
    find(node.props.children, tag, out);
  }
  return out;
}
const tree = ChoiceView({ reads: { view: "release.Repositories" }, ...props });
const options = find(tree, "option", []).slice(1).map((o) => [o.props.value, o.props.children]);
assert.deepStrictEqual(options, [["repo-1", "repo-1"], ["repo-2", "repo-2"]],
  `options carry the rows' repository_id and a readable label: ${JSON.stringify(options)}`);
const [select] = find(tree, "select", []);
select.props.onChange({ target: { value: "repo-1" } });
assert.deepStrictEqual(picked, [["draft.repository_id", "repo-1"]],
  `picking sends the row's repository_id: ${JSON.stringify(picked)}`);
"#;

/// Generates the document into `<scratch>/app`.
fn generate(name: &str) -> (PathBuf, ess_ui_react::GeneratedFiles) {
    let dir = scratch(name);
    write(&dir.join("repositories.yaml"), FIXTURE);
    let document = ess_ui::load_str(DOCUMENT).expect("the document loads");
    let project = dir.join("app");
    let files = ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    (project, files)
}

/// The generated page passes the field's name as the key a row's value is read from.
#[test]
fn a_field_choice_is_told_to_take_its_value_from_the_rows_field() {
    let (project, files) = generate("emit");
    let (ok, errors) = run(
        "tsc",
        &["-p", "tsconfig.offline.json", "--noEmit"],
        &project,
    );
    assert!(ok, "the generated project type-checks:\n{errors}");
    let page = files
        .files
        .values()
        .find(|text| text.contains("<ChoiceView"))
        .expect("a generated file renders the choice");
    let element = &page[page.find("<ChoiceView").expect("the element")..];
    let element = &element[..element.find("/>").expect("the element closes")];
    assert!(
        element.contains(r#"valueKey={"repository_id"}"#),
        "the choice is told its rows' value key:\n{element}"
    );
}

/// The runtime choice reads that key from each row: rows with `repository_id` and no `id` send
/// the chosen row's `repository_id`, and each option's label is readable.
#[test]
fn a_choice_over_rows_without_id_sends_the_rows_field_value_and_shows_a_readable_label() {
    let (project, _) = generate("runtime");
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
    let runtime = project.join("out/src/runtime");
    assert!(
        runtime.join("composites/choice.js").exists(),
        "tsc emitted the choice: {compiled}"
    );
    write(&project.join("out/package.json"), r#"{"type":"commonjs"}"#);
    write(&runtime.join("core.js"), CORE_STUB);
    write(&runtime.join("data.js"), DATA_STUB);
    write(&runtime.join("expr.js"), EXPR_STUB);
    write(&project.join("choice.cjs"), SCRIPT);
    let props = serde_json::json!({
        "binds": "draft.repository_id",
        "valueKey": "repository_id",
    });
    let (ok, output) = run("node", &["choice.cjs", &props.to_string()], &project);
    assert!(ok, "{output}");
}
