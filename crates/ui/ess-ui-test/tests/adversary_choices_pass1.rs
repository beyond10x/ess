//! Adversary pass 1 for beyond10x/ess#330: a document whose choice `options` name a model enum
//! is admitted by `ess ui check --model` and by every bound generator, and its refusal without a
//! model tells the author to "load the document with its model (`--model`)". `ess ui test` has
//! no `--model` (`TestArgs`: `--path`, tests, `--format`, `--playwright`), so the same document
//! cannot be tested at all: not one test of the file runs, whichever page it opens.

use std::path::{Path, PathBuf};

const DOCUMENT: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.list
  sections: [{name: all, pages: [releases.list, releases.risk]}]
pages:
  releases.list:
    kind: detail_page
    title: Releases
    sections:
      - {name: summary, component: collection, reads: {placeholder: release.Repositories, fixture: repositories.yaml}, columns: [location]}
  releases.risk:
    kind: detail_page
    title: Risk
    state:
      risk: {type: string, class: page_state, store: url}
    sections:
      - {name: risk, component: choice, options: objective.RiskLevel, binds: state.risk}
";

const FIXTURE: &str = r"
view: release.Repositories
rows:
  - {repository_id: repo-1, location: example/one}
";

fn document() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-test-adversary-choices-pass1");
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("repositories.yaml"), FIXTURE).expect("the fixture is written");
    let path = dir.join("ui.yaml");
    std::fs::write(&path, DOCUMENT).expect("the document is written");
    path
}

/// A test that never touches the model-enum choice: it opens the other page and reads a fixture.
#[test]
fn a_document_naming_a_model_enum_in_its_options_can_be_tested() {
    let document = document();
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n\
         - name: list\n  steps:\n\
         \x20 - open: releases.list\n\
         \x20 - expect: {{at: pages/releases.list/sections/summary, text: example/one}}\n",
        document.display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("risk.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| {
        panic!(
            "`ess ui test` cannot start a run on a document `ess ui check --model` admits, and \
             names a `--model` it does not take: {error}"
        )
    });
    assert_eq!(report.tests.len(), 1, "{report:?}");
}
