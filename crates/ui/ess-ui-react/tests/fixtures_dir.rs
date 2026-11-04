//! A placeholder read takes its fixture from the declared `fixtures.dir`
//! (story:a-value-renders-in-the-form-its-field-declares, A41): its `fixture:` names a file in
//! that directory, as a view or script file of the fixture index does; a path that already starts
//! with the declared directory names the same file; without a declared directory it is relative
//! to the document.

use std::path::{Path, PathBuf};

fn document(fixtures: &str, fixture: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
{fixtures}shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: plan
  sections: [{{name: all, pages: [plan]}}]
pages:
  plan:
    kind: list_page
    title: Plan
    sections:
      - name: quarters
        component: collection
        reads: {{placeholder: t.Quarters, fixture: {fixture}}}
        columns: [quarter]
"
    )
}

const ROWS: &str =
    "views:\n  t.Quarters:\n    rows:\n      - {id: q-1, quarter: from-the-fixture-dir}\n";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-fixtures-dir")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

/// Generates `text` from `dir` and returns its `src/fixtures.ts`.
fn fixtures_ts(dir: &Path, text: &str) -> Result<String, String> {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let files = ess_ui_react::render(&document, dir).map_err(|error| error.to_string())?;
    Ok(files
        .files
        .iter()
        .find(|(path, _)| path.as_str() == "src/fixtures.ts")
        .map(|(_, text)| text.clone())
        .expect("the project has src/fixtures.ts"))
}

#[test]
fn a_placeholder_read_takes_its_fixture_from_fixtures_dir() {
    let dir = scratch("dir");
    std::fs::create_dir_all(dir.join("ui/data")).expect("the fixture dir is created");
    std::fs::write(dir.join("ui/data/quarters.yaml"), ROWS).expect("the fixture is written");

    let fixtures = "fixtures: {dir: ui/data, views: {}}\n";
    let ts = fixtures_ts(&dir, &document(fixtures, "quarters.yaml"))
        .unwrap_or_else(|error| panic!("the placeholder reads from `fixtures.dir`: {error}"));
    assert!(ts.contains("from-the-fixture-dir"), "{ts}");

    let prefixed =
        fixtures_ts(&dir, &document(fixtures, "ui/data/quarters.yaml")).unwrap_or_else(|error| {
            panic!("a path under the declared dir names the same file: {error}")
        });
    assert_eq!(prefixed, ts);

    // A file beside the document, not in the declared dir, is not read.
    std::fs::write(dir.join("beside.yaml"), ROWS).expect("the fixture is written");
    assert!(
        fixtures_ts(&dir, &document(fixtures, "beside.yaml")).is_err(),
        "a placeholder fixture outside `fixtures.dir` is not found"
    );
}

#[test]
fn without_a_fixtures_dir_a_placeholder_fixture_is_relative_to_the_document() {
    let dir = scratch("no-dir");
    std::fs::create_dir_all(dir.join("data")).expect("the fixture dir is created");
    std::fs::write(dir.join("data/quarters.yaml"), ROWS).expect("the fixture is written");
    for fixtures in ["", "fixtures: {views: {}}\n"] {
        let ts = fixtures_ts(&dir, &document(fixtures, "data/quarters.yaml"))
            .unwrap_or_else(|error| panic!("{fixtures:?}: {error}"));
        assert!(ts.contains("from-the-fixture-dir"), "{fixtures:?}: {ts}");
    }
}
