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
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String> {
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
