//! Adversary pass on the wave's ride-along React runtime pull requests (beyond10x/ess#745 `contains`,
//! #746 tag `suggests`): each case drives the generated runtime under `node` and compares it with
//! what the shared Rust library, the terminal and the checker decide for the same document.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::NodeRef;
use ess_ui_check::{check_source, Options};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w089-ui")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn generate(name: &str, yaml: &str) -> (PathBuf, String) {
    let out = scratch(name);
    let document =
        ess_ui::load_str(yaml).unwrap_or_else(|error| panic!("the document loads: {error}"));
    let files = ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    let pages: String = files
        .files
        .iter()
        .filter(|(path, _)| !path.starts_with("src/runtime/"))
        .map(|(_, text)| text.as_str())
        .collect();
    (out, pages)
}

fn lookup(scope: &ess_yaml::Value) -> impl Fn(&[String]) -> ess_yaml::Value + '_ {
    move |path: &[String]| {
        let mut value = scope;
        for segment in path {
            value = &value[segment.as_str()];
        }
        value.clone()
    }
}

// ── #745: a toggle state written with `contains` ─────────────────────────────────────────────

const TOGGLES: &str = r#"
format: ess-ui/1
app: flags
model: flags.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: flags.list
  sections: [{name: all, pages: [flags.list]}]
pages:
  flags.list:
    kind: detail_page
    title: Flags
    sections:
      - name: list
        component: collection
        reads: {view: flags.All}
        columns: [title]
        row_actions:
          - {name: beta, as: toggle, label: Beta, does: flags.SetFlag, bind: {id: row.id, beta: "not (row.title contains 'et')"}}
"#;

/// `contains` is a filter operator only, but #745 also taught the toggle state reader
/// (`ess_ui::toggle_state`, the terminal's `toggle_on`) to read it, so a toggle whose `not` bind
/// entry is a `contains` now names a state. The React runtime evaluates that state as a display
/// expression, where `contains` does not parse, so the state is literal text and every row's
/// toggle is drawn on. Before #745 no renderer named such a state and `ess ui check` warned
/// `toggle_state_unknown`; now the check is silent and the two renderers disagree.
#[test]
fn a_toggle_state_written_with_contains_is_drawn_alike_by_the_terminal_and_react() {
    let document = ess_ui::load_str(TOGGLES).expect("the document loads");
    let action = document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            NodeRef::Action(action) if action.name == "beta" => Some(action),
            _ => None,
        })
        .expect("the toggle");
    let state = ess_ui::toggle_state(action).expect("#745: the library names the state");
    assert_eq!(state.expr.0, "row.title contains 'et'");

    // The terminal (`App::toggle_on`): the state read in the filter grammar, judged truthy.
    let predicate = ess_ui::filter::parse(&state.expr.0)
        .filter(ess_ui::filter::Predicate::allowed_roots)
        .expect("the terminal reads the state in the filter grammar");
    let terminal: Vec<bool> = ["Alpha", "Beta", "Gamma"]
        .iter()
        .map(|title| {
            let scope: ess_yaml::Value =
                ess_yaml::from_str(&format!("{{row: {{title: {title}}}}}")).unwrap();
            let resolve = lookup(&scope);
            ess_ui::filter::truthy(&predicate.evaluate(&resolve))
        })
        .collect();
    assert_eq!(terminal, [false, true, false]);

    let (out, _) = generate("toggle", TOGGLES);
    support::compile(&out, &["pages/FlagsList.tsx"]);
    support::node(
        &out,
        "toggle",
        r"
browser('/');
const { setDataAdapter } = out('runtime/data');
const { FlagsListPage } = out('pages/FlagsList');
const rows = [{ id: 'f-1', title: 'Alpha' }, { id: 'f-2', title: 'Beta' }, { id: 'f-3', title: 'Gamma' }];
setDataAdapter({
  read: async () => ({ rows: rows.map((row) => ({ ...row })), total: rows.length }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(FlagsListPage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
(async () => {
  for (let i = 0; i < 4; i++) { root.settle(); await tick(); }
  root.settle();
  const checked = root.find((node) => node.tag === 'input' && node.props.type === 'checkbox' && node.props.checked !== undefined)
    .map((node) => node.props.checked);
  assert.deepStrictEqual(checked, [false, true, false],
    '#745 defect: the terminal reads the toggle state `row.title contains et` in the filter grammar (only Beta on), but the React runtime evaluates it as a display expression in which `contains` does not parse, so the state is literal text and every toggle is drawn on');
})().catch((error) => { console.error(error); process.exitCode = 1; });
",
    );
}

// ── #745: `contains` splits the search into words alike in Rust and in the browser ────────────

/// `ess_ui::filter` splits the search with `split_whitespace` (Unicode `White_Space`); `expr.ts`
/// splits with `/\s+/`. The two sets differ: U+0085 (NEL) is whitespace to Rust and not to
/// ECMAScript, U+FEFF (BOM) the other way round. The terminal and the browser then keep different
/// rows for one search text.
#[test]
fn contains_splits_the_search_into_the_same_words_in_rust_and_react() {
    const DOCUMENT: &str = r"format: ess-ui/1
app: filters
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: Filters
    sections:
      - {name: list, component: collection, reads: {view: t.All, filter: row.text contains state.q}, columns: [id]}
";
    let (out, _) = generate("words", DOCUMENT);
    support::compile(&out, &["runtime/expr.ts"]);
    let filter = "row.text contains state.q";
    let scopes = [
        serde_json::json!({"row": {"text": "b a"}, "state": {"q": "a\u{85}b"}}),
        serde_json::json!({"row": {"text": "x"}, "state": {"q": "\u{feff}"}}),
        serde_json::json!({"row": {"text": "b a"}, "state": {"q": "a b"}}),
    ];
    let expected: Vec<bool> = scopes
        .iter()
        .map(|scope| {
            let scope: ess_yaml::Value = ess_yaml::to_value(scope).unwrap();
            let resolve = lookup(&scope);
            ess_ui::filter::keeps(filter, &resolve)
        })
        .collect();
    support::node(
        &out,
        "words",
        &format!(
            r"
const {{ keeps }} = out('runtime/expr');
const scopes = {scopes};
assert.deepStrictEqual(scopes.map((scope) => keeps({filter:?}, scope)), {expected},
  '#745 defect: `contains` splits the search on different whitespace in the React runtime (/\\s+/) and in ess_ui::filter (split_whitespace), so a search holding U+0085 or U+FEFF keeps different rows in the browser and the terminal');
",
            scopes = serde_json::to_string(&scopes).unwrap(),
            expected = serde_json::to_string(&expected).unwrap(),
        ),
    );
}

// ── #746: a suggestions read with a client filter ─────────────────────────────────────────────

const SUGGESTS: &str = r#"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: runs
  sections: [{name: all, pages: [runs]}]
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: add
        component: form
        does: t.AddRun
        fields:
          - {field: tags, label: Tags, as: tags, suggests: {reads: {view: t.Tags, filter: "row.archived == false"}, value: name}}
"#;

/// `suggests.reads` is a `Reads`, so it takes a client row `filter`, and the generated page passes
/// that filter to the field. But `TagsInput` reads with `useRead` and never applies it, so a row the
/// filter hides is suggested; and no filter check visits `…/suggests/reads/filter`, so a filter
/// that does not even parse there is not reported. A client filter must fail closed: an unread
/// predicate may never reveal a row.
#[test]
fn a_filter_on_a_suggestions_read_is_checked_as_a_client_filter() {
    let broken = SUGGESTS.replace("row.archived == false", "row.archived ==");
    let report = check_source(
        &broken,
        "ui.yaml",
        Path::new("."),
        None,
        &Options::default(),
    );
    let path = "pages/runs/sections/add/fields/tags/suggests/reads/filter";
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.check.starts_with("filter_") && finding.path == path),
        "#746 defect: a filter on a tags field's suggestions read is checked by no filter rule, so \
         `row.archived ==` at {path} is not reported: {:#?}",
        report.findings
    );
}

#[test]
fn a_suggestions_read_filter_hides_the_rows_it_excludes() {
    let (out, pages) = generate("suggests", SUGGESTS);
    assert!(
        pages.contains(r#"filter: "row.archived == false""#),
        "the page passes the suggestions read its filter:\n{pages}"
    );
    support::compile(&out, &["runtime/fields.tsx"]);
    support::node(
        &out,
        "suggests",
        r##"
const { FieldInput } = out("runtime/fields");
const { ScopeLayer } = out("runtime/core");
const { setDataAdapter } = out("runtime/data");
const rows = [{ name: "vip", archived: false }, { name: "legacy", archived: true }];
setDataAdapter({
  read: async (request) => ({ rows: request.view === "t.Tags" ? rows : [], total: rows.length }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
function Host(props) {
  const [draft, setDraft] = React.useState(props.draft);
  return jsx(ScopeLayer, { values: { draft }, setters: { draft: setDraft }, children: jsx(FieldInput, { field: props.field }) });
}
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
(async () => {
  const field = { "data-ui-path": "pages/runs/sections/add/fields/tags", field: "tags", name: "tags", as: "tags", suggests: { reads: { view: "t.Tags", filter: "row.archived == false" }, value: "name" } };
  const root = React.__root(jsx(Host, { field, draft: { tags: [] } }));
  for (let i = 0; i < 4; i++) { root.settle(); await tick(); }
  root.settle();
  const offered = root.find((n) => n.tag === "button" && /\bui-tag-suggestion\b/.test(String(n.props.className))).map(textOf);
  assert.deepStrictEqual(offered, ["vip"],
    "#746 defect: TagsInput ignores the suggestions read's client filter `row.archived == false` and suggests the archived row it excludes");
})().catch((error) => { console.error(error); process.exitCode = 1; });
"##,
    );
}
