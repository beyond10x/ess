//! Value labels in the terminal (story:a-value-shows-its-declared-label-beside-its-tone): a badge
//! column and a badge primitive show the label the document declares for their value; a value the
//! map does not name shows as it did.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r#"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {views: {t.Runs: data.yaml, t.ById: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: runs
  sections: [{name: all, pages: [runs, run]}]
label_maps:
  run_state: {in_progress: In progress, no_answer: No answer}
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: {view: t.Runs}
        columns:
          - {field: title}
          - {field: state, as: badge, labels: run_state}
          - {field: enabled, as: badge, labels: {"true": Active, "false": Inactive}}
  run:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        component: record
        reads: {view: t.ById}
        item:
          - {name: state, primitive: badge, field: state, labels: {in_progress: Running now}}
"#;

const DATA: &str = r"
views:
  t.Runs:
    rows:
      - {id: r-1, title: First, state: in_progress, enabled: true}
      - {id: r-2, title: Second, state: queued, enabled: false}
  t.ById:
    rows:
      - {id: r-1, title: First, state: in_progress, enabled: true}
";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-value-labels")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

fn screen(test: &str, page: &str) -> String {
    let base = dir(test);
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page(page, &[]);
    app.render_text(140, 40)
}

#[test]
fn a_badge_column_shows_the_label() {
    let screen = screen("column", "runs");
    for shown in ["[In progress]", "[Active]", "[Inactive]", "[queued]"] {
        assert!(
            screen.contains(shown),
            "the column shows {shown}:\n{screen}"
        );
    }
    for raw in ["[in_progress]", "[true]", "[false]"] {
        assert!(
            !screen.contains(raw),
            "a labelled value is not shown raw ({raw}):\n{screen}"
        );
    }
}

#[test]
fn a_badge_primitive_shows_the_label() {
    let screen = screen("primitive", "run");
    assert!(screen.contains("[Running now]"), "{screen}");
    assert!(!screen.contains("[in_progress]"), "{screen}");
}
