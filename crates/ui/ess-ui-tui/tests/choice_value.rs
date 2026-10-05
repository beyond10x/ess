//! A form field drawn as a choice over a view whose rows carry no `id`: the option value is the
//! row's key named like the field, and its label is readable.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Options, ReadRequest, ReadResult};
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

type Sent = Rc<RefCell<Vec<(String, BTreeMap<String, Value>)>>>;

/// The fixture adapter, with every command it runs copied out for the test to read.
struct Recording {
    inner: FixtureAdapter,
    sent: Sent,
}

impl DataAdapter for Recording {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        self.inner.read(request)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        self.sent
            .borrow_mut()
            .push((command.to_owned(), input.clone()));
        self.inner.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-choice")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    dir
}

#[test]
fn a_choice_over_rows_without_id_sends_the_rows_field_value_and_shows_a_readable_label() {
    let base = dir("repository");
    std::fs::write(base.join("repositories.yaml"), FIXTURE).expect("the fixture is written");
    let document = ess_ui::load_str(DOCUMENT).expect("the document loads");
    let (inner, scripts) = FixtureAdapter::load(&document, &base, None).expect("the fixtures load");
    let sent = Sent::default();
    let adapter = Recording {
        inner,
        sent: Rc::clone(&sent),
    };
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        scripts,
        Options::new(base.join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));

    app.focus_section("form");
    app.keys("<enter>");
    let screen = app.render_text(120, 24);
    assert!(
        screen.contains("‹repo-1› of repo-1/repo-2"),
        "the field shows the chosen repository_id and readable option labels:\n{screen}"
    );

    app.keys("<c-s>");
    let sent = sent.borrow();
    let (command, input) = sent.last().expect("a command is sent");
    assert_eq!(command, "release.Cut");
    assert_eq!(
        input.get("repository_id"),
        Some(&Value::String("repo-1".into())),
        "the command carries the chosen row's repository_id, not the whole row: {input:?}"
    );
}

// ── beyond10x/ess#328: named value and label fields, the view's identity ────────────────────

/// Answers every read with `rows` and records every command sent.
struct Table {
    rows: Vec<Value>,
    sent: Sent,
}

impl DataAdapter for Table {
    fn read(&self, _: &ReadRequest) -> Result<ReadResult, String> {
        Ok(ReadResult {
            rows: self.rows.clone(),
            total: None,
        })
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        self.sent
            .borrow_mut()
            .push((command.to_owned(), input.clone()));
        ess_ui::binding::Answer::Accepted
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

const REPOSITORIES: &str = "[{repository_id: repo-1, location: example/one, id: wrong-1}, \
                            {repository_id: repo-2, location: example/two, id: wrong-2}]";

/// A form whose field `selected_repository` is a choice over `release.Repositories` written as
/// `choice`, and a filter bar whose one choice writes `state.repository`, written as `bar`.
fn projected(choice: &str, bar: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: releases.new
  sections: [{{name: all, pages: [releases.new]}}]
pages:
  releases.new:
    kind: form_page
    title: New release
    state:
      repository: {{type: string, class: page_state, store: url}}
    sections:
      - name: form
        component: form
        does: release.Cut
        fields:
          - field: selected_repository
            as: choice
            choice: {{component: choice, reads: {{view: release.Repositories}}{choice}}}
      - name: bar
        component: filter_bar
        binds: [state.repository]
        choices:
          - {{name: repository, component: choice, reads: {{view: release.Repositories}}{bar}}}
"
    )
}

/// Numbers the apps [`app_over`] opens, so each has its own state directory: the tests run in
/// parallel.
static APPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn app_over(text: &str, rows: &str, binding: Option<ess_ui::binding::Binding>) -> (App, Sent) {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let sent = Sent::default();
    let adapter = Box::new(Table {
        rows: serde_yaml::from_str(rows).expect("the rows parse"),
        sent: Rc::clone(&sent),
    });
    let serial = APPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let options =
        Options::new(dir(&format!("projected-{}-{serial}", std::process::id())).join("state"));
    let app = match binding {
        None => App::with_adapter(document, adapter, Vec::new(), options),
        Some(binding) => App::bound(document, adapter, binding, options),
    }
    .unwrap_or_else(|error| panic!("{error}"));
    (app, sent)
}

/// Picks the first option of the form field and submits: what the command carried.
fn submitted(app: &mut App, sent: &Sent) -> Option<Value> {
    app.focus_section("form");
    app.keys("<enter>");
    app.keys("<c-s>");
    let sent = sent.borrow();
    let (command, input) = sent.last().expect("a command is sent");
    assert_eq!(command, "release.Cut");
    input.get("selected_repository").cloned()
}

#[test]
fn an_explicit_value_and_label_send_the_named_field_into_a_differently_named_input() {
    let (mut app, sent) = app_over(
        &projected(", value: repository_id, label: location", ""),
        REPOSITORIES,
        None,
    );
    app.focus_section("form");
    app.keys("<enter>");
    let screen = app.render_text(120, 24);
    assert!(
        screen.contains("‹repo-1› of example/one/example/two"),
        "the field holds the row's repository_id and shows each row's location:\n{screen}"
    );
    app.keys("<c-s>");
    let sent = sent.borrow();
    let (_, input) = sent.last().expect("a command is sent");
    assert_eq!(
        input.get("selected_repository"),
        Some(&Value::String("repo-1".into())),
        "the command carries repository_id, not `id`: {input:?}"
    );
}

#[test]
fn the_reads_key_names_the_value_when_no_value_is_written() {
    let text = projected("", "").replacen(
        "reads: {view: release.Repositories}}",
        "reads: {view: release.Repositories, key: repository_id}}",
        1,
    );
    let (mut app, sent) = app_over(&text, REPOSITORIES, None);
    assert_eq!(
        submitted(&mut app, &sent),
        Some(Value::String("repo-1".into()))
    );
}

/// Bound to a served surface, a choice without `value` or `key` takes the view's identity field
/// from the binding; without the binding the same document keeps the legacy `id`.
#[test]
fn a_bound_choice_takes_its_value_from_the_views_identity() {
    let binding: ess_ui::binding::Binding = serde_json::from_str(
        r#"{"system": "release", "components": {"svc": {"views": {"release.Repositories":
            {"path": "/repositories", "params": [], "identity": "repository_id"}},
            "commands": {"release.Cut": {"path": "/cut", "body_required": true, "errors": {}}}}},
            "names": {}}"#,
    )
    .expect("the binding reads");
    let (mut app, sent) = app_over(&projected("", ""), REPOSITORIES, Some(binding));
    assert_eq!(
        submitted(&mut app, &sent),
        Some(Value::String("repo-1".into())),
        "the identity, not `id`"
    );
    let (mut app, sent) = app_over(&projected("", ""), REPOSITORIES, None);
    assert_eq!(
        submitted(&mut app, &sent),
        Some(Value::String("wrong-1".into())),
        "without a binding the legacy fallback is unchanged"
    );
}

#[test]
fn a_typed_value_is_sent_with_its_type() {
    let (mut app, sent) = app_over(
        &projected(", value: repository_no, label: location", ""),
        "[{repository_no: 7, location: seven}, {repository_no: 8, location: eight}]",
        None,
    );
    assert_eq!(submitted(&mut app, &sent), Some(Value::from(7)));
}

/// A row without the `value` field offers no option; a row without the `label` field shows its
/// value.
#[test]
fn a_row_missing_the_value_field_offers_no_option() {
    let (mut app, _) = app_over(
        &projected(", value: repository_id, label: location", ""),
        "[{location: nowhere, id: wrong}, {repository_id: repo-2}]",
        None,
    );
    app.focus_section("form");
    app.keys("<enter>");
    let screen = app.render_text(120, 24);
    assert!(screen.contains("‹repo-2› of repo-2]"), "{screen}");
}

/// A filter bar's choice projects the same way: it writes the row's `repository_id` into its
/// state and shows each row's `location`.
#[test]
fn a_filter_bar_choice_projects_its_value_and_label() {
    let (mut app, _) = app_over(
        &projected("", ", value: repository_id, label: location"),
        REPOSITORIES,
        None,
    );
    app.focus_section("bar");
    app.keys("<space>");
    let screen = app.render_text(120, 24);
    assert!(
        screen.contains("repository: (•) example/one ( ) example/two"),
        "{screen}"
    );
    assert!(
        screen.contains("/releases.new?repository=repo-1"),
        "the state holds repository_id, not `id`:\n{screen}"
    );
}

// ── beyond10x/ess#330: options naming a model enum ──────────────────────────────────────────

/// The generated terminal crate loads its document with the enums its binding carries, and is
/// refused, before anything is written, when the binding does not carry one the options name.
#[test]
fn a_generated_crate_lists_a_model_enums_variants_from_its_binding() {
    let text = projected("", "").replace(
        "      - name: bar\n",
        "      - {name: risk, component: choice, options: objective.RiskLevel, binds: state.repository}\n      - name: bar\n",
    );
    let mut binding: ess_ui::binding::Binding = serde_json::from_str(
        r#"{"system": "release", "components": {"svc": {"views": {"release.Repositories":
            {"path": "/repositories", "params": []}},
            "commands": {"release.Cut": {"path": "/cut", "body_required": true, "errors": {}}}}},
            "names": {"release.Repositories": "release.Repositories", "release.Cut": "release.Cut"}}"#,
    )
    .expect("the binding reads");
    let refused = ess_ui_tui::generate::render(&text, &binding).expect_err("no enum carried");
    assert!(
        refused.to_string().contains("objective.RiskLevel"),
        "{refused}"
    );
    binding.names.insert(
        "objective.RiskLevel".to_owned(),
        "release.objective.RiskLevel".to_owned(),
    );
    binding.enums.insert(
        "release.objective.RiskLevel".to_owned(),
        vec![ess_ui::binding::EnumVariant {
            value: "low".to_owned(),
            label: "Low".to_owned(),
        }],
    );
    let files = ess_ui_tui::generate::render(&text, &binding)
        .unwrap_or_else(|error| panic!("the crate renders: {error}"));
    assert!(files["src/binding.rs"].contains("release.objective.RiskLevel"));
}
