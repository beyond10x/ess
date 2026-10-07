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

// ── beyond10x/ess#328: named value and label fields, the view's identity ────────────────────

/// Four choices over one view: a form field's and a filter bar's with `value` and `label`, a
/// standalone one whose read names its `key`, and a plain one.
const PROJECTED: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
fixtures: {views: {release.Repositories: repositories.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.new
  sections: [{name: all, pages: [releases.new]}]
pages:
  releases.new:
    kind: form_page
    title: New release
    state:
      repository: {type: string, class: page_state, store: url}
    sections:
      - name: form
        component: form
        does: release.Cut
        fields:
          - field: selected_repository
            as: choice
            choice: {component: choice, reads: {view: release.Repositories}, value: repository_id, label: location}
      - name: bar
        component: filter_bar
        binds: [state.repository]
        choices:
          - {name: repository, component: choice, reads: {view: release.Repositories}, value: repository_id, label: location}
      - {name: pick, component: choice, reads: {view: release.Repositories, key: repository_id}, binds: state.repository}
      - {name: plain, component: choice, reads: {view: release.Repositories}, binds: state.repository}
";

/// A binding whose `release.Repositories` rows are identified by `repository_id`.
fn identity_binding() -> ess_ui::binding::Binding {
    serde_json::from_str(
        r#"{"system": "release", "components": {"svc": {"views": {"release.Repositories":
            {"path": "/repositories", "params": [], "identity": "repository_id"}},
            "commands": {"release.Cut": {"path": "/cut", "body_required": true, "errors": {}}}}},
            "names": {"release.Repositories": "release.Repositories", "release.Cut": "release.Cut"}}"#,
    )
    .expect("the binding reads")
}

fn occurrences(files: &ess_ui_react::GeneratedFiles, needle: &str) -> usize {
    files
        .files
        .values()
        .map(|text| text.matches(needle).count())
        .sum()
}

/// Every choice is told the field its value comes from and the one its label shows; bound, every
/// choice over the view is also told the view's identity, and the project still type-checks.
#[test]
fn every_choice_over_a_view_is_told_its_value_label_and_identity_fields() {
    let dir = scratch("projected");
    write(&dir.join("repositories.yaml"), FIXTURE);
    let document = ess_ui::load_str(PROJECTED).unwrap_or_else(|error| panic!("{error}"));
    let plain = ess_ui_react::generate(&document, &dir, &dir.join("plain"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(occurrences(&plain, "<ChoiceView"), 4);
    assert_eq!(occurrences(&plain, r#"valueField={"repository_id"}"#), 3);
    assert_eq!(occurrences(&plain, r#"labelField={"location"}"#), 2);
    assert_eq!(
        occurrences(&plain, "identity={"),
        0,
        "no binding, no identity"
    );

    let binding = identity_binding();
    let bound = ess_ui_react::generate_bound(&document, &dir, &dir.join("bound"), Some(&binding))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(occurrences(&bound, r#"identity={"repository_id"}"#), 4);
    let (ok, errors) = run(
        "tsc",
        &["-p", "tsconfig.offline.json", "--noEmit"],
        &dir.join("bound"),
    );
    assert!(ok, "the bound project type-checks:\n{errors}");
}

/// Renders the choice with `case.props` over `case.rows`, checks its options, picks `case.pick`
/// and checks what it wrote.
const CASE: &str = r#""use strict";
const assert = require("assert");
const { ChoiceView } = require("./out/src/runtime/composites/choice");
const c = JSON.parse(process.argv[2]);
const picked = [];
globalThis.__scope = { values: {}, set: (path, value) => picked.push([path, value]) };
globalThis.__rows = c.rows;
function find(node, tag, out) {
  if (Array.isArray(node)) { node.forEach((n) => find(n, tag, out)); return out; }
  if (node && typeof node === "object" && node.props) {
    if (node.type === tag) out.push(node);
    find(node.props.children, tag, out);
  }
  return out;
}
const tree = ChoiceView({ reads: { view: "release.Repositories" }, binds: "draft.selected_repository", ...c.props });
const options = find(tree, "option", []).slice(1).map((o) => [o.props.value, o.props.children]);
assert.deepStrictEqual(options, c.options, `${c.name}: options ${JSON.stringify(options)}`);
const [select] = find(tree, "select", []);
select.props.onChange({ target: { value: c.pick } });
assert.deepStrictEqual(picked, [["draft.selected_repository", c.picked]], `${c.name}: picked ${JSON.stringify(picked)}`);
"#;

#[test]
fn the_runtime_choice_projects_value_and_label_as_the_contract_says() {
    let (project, _) = generate("projection-runtime");
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
    write(&project.join("case.cjs"), CASE);
    let repositories = serde_json::json!([
        {"repository_id": "repo-1", "location": "example/one", "id": "wrong-1", "uid": "u-1"},
        {"repository_id": "repo-2", "location": "example/two", "id": "wrong-2", "uid": "u-2"},
    ]);
    let cases = [
        serde_json::json!({
            "name": "explicit value and label beat the field, the identity and id",
            "props": {"valueField": "repository_id", "labelField": "location",
                      "valueKey": "selected_repository", "identity": "uid"},
            "rows": repositories,
            "options": [["repo-1", "example/one"], ["repo-2", "example/two"]],
            "pick": "repo-2", "picked": "repo-2",
        }),
        serde_json::json!({
            "name": "the view's identity is the default value",
            "props": {"valueKey": "selected_repository", "identity": "repository_id"},
            "rows": repositories,
            "options": [["repo-1", "repo-1"], ["repo-2", "repo-2"]],
            "pick": "repo-1", "picked": "repo-1",
        }),
        serde_json::json!({
            "name": "a typed value keeps its type",
            "props": {"valueField": "repository_no", "labelField": "location"},
            "rows": [{"repository_no": 7, "location": "seven"}, {"repository_no": 8, "location": "eight"}],
            "options": [["7", "seven"], ["8", "eight"]],
            "pick": "8", "picked": 8,
        }),
        serde_json::json!({
            "name": "a row without the value field offers nothing; one without the label shows its value",
            "props": {"valueField": "repository_id", "labelField": "location"},
            "rows": [{"location": "nowhere", "id": "wrong"}, {"repository_id": "repo-2"}],
            "options": [["repo-2", "repo-2"]],
            "pick": "repo-2", "picked": "repo-2",
        }),
        serde_json::json!({
            "name": "without a projection or identity the legacy id fallback is unchanged",
            "props": {"valueKey": "selected_repository"},
            "rows": repositories,
            "options": [["wrong-1", "wrong-1"], ["wrong-2", "wrong-2"]],
            "pick": "wrong-1", "picked": "wrong-1",
        }),
    ];
    for case in cases {
        let (ok, output) = run("node", &["case.cjs", &case.to_string()], &project);
        assert!(ok, "{}: {output}", case["name"]);
    }
}

// ── beyond10x/ess#330: options naming a model enum ──────────────────────────────────────────

/// `ess generate ui --target react --model` loads the document with the enums the binding
/// carries; without a model, the options naming one are refused.
#[test]
fn a_bound_project_lists_a_model_enums_variants() {
    let dir = scratch("model-enum");
    write(&dir.join("repositories.yaml"), FIXTURE);
    let path = dir.join("ui.yaml");
    write(
        &path,
        &PROJECTED.replace(
            "      - {name: plain,",
            "      - {name: risk, component: choice, options: objective.RiskLevel, binds: state.repository}\n      - {name: plain,",
        ),
    );
    let mut binding = identity_binding();
    binding.names.insert(
        "objective.RiskLevel".to_owned(),
        "release.objective.RiskLevel".to_owned(),
    );
    binding.enums.insert(
        "release.objective.RiskLevel".to_owned(),
        ["low", "high"]
            .map(|value| ess_ui::binding::EnumVariant {
                value: value.to_owned(),
                label: value.to_uppercase(),
            })
            .to_vec(),
    );
    let args = ess_ui_react::ReactArgs {
        path: path.clone(),
        out: dir.join("app"),
        model: Some(dir.join("model")),
    };
    ess_ui_react::run(&args, Some(&binding)).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(dir.join("app/src/pages/ReleasesNew.tsx"))
        .expect("the page is written");
    assert!(
        page.contains(r#""low""#) && page.contains(r#""LOW""#) && page.contains(r#""HIGH""#),
        "{page}"
    );
    let unbound = ess_ui_react::ReactArgs {
        path,
        out: dir.join("unbound"),
        model: None,
    };
    let refused = ess_ui_react::run(&unbound, None).expect_err("no model, no enum");
    assert!(
        refused.to_string().contains("no model was given"),
        "{refused}"
    );
}
