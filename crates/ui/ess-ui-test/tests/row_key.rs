//! beyond10x/ess#320: a section without a live channel, over a view keyed by a field other than
//! `id`, keys its rows by the read's `key`, so `ess ui test` row paths and row counts address them.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const FIXTURE: &str = "views:\n  factory.Objectives:\n    rows:\n      \
                       - {objective_id: obj-0001, title: Ship}\n      \
                       - {objective_id: obj-0002, title: Test}\n";

const SCRIPT: &str = "channel: objectives\nevents:\n  \
                      - {at: 1s, event: factory.ObjectiveChanged, payload: {objective_id: obj-0002, title: Tested}}\n";

fn document(name: &str, reads: &str, live: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-row-key")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("objectives.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(dir.join("script.yaml"), SCRIPT).expect("the script is written");
    let channel = if live.is_empty() {
        String::new()
    } else {
        "channels:\n  objectives:\n    carries: {events: [factory.ObjectiveChanged]}\n    \
         direction: server_to_client\n    delivery: every_event\n    resume: refetch\n"
            .to_owned()
    };
    std::fs::write(
        dir.join("ui.yaml"),
        format!(
            "format: ess-ui/1
app: factory
model: factory.system
placement_profile: fat
fixtures: {{views: {{factory.Objectives: objectives.yaml}}, scripts: {{objectives: script.yaml}}}}
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
navigation:
  home: objectives
  sections: [{{name: all, pages: [objectives]}}]
{channel}pages:
  objectives:
    kind: detail_page
    title: Objectives
    sections:
      - name: objectives
        component: collection
        reads: {reads}
        columns: [title]
{live}"
        ),
    )
    .expect("the document is written");
    dir.join("ui.yaml")
}

fn run(document: &Path, steps: &[&str]) -> Outcome {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: row key\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("row_key.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

const STEPS: &[&str] = &[
    "open: objectives",
    "expect: {at: pages/objectives/sections/objectives, rows: [obj-0001, obj-0002]}",
    "expect: {at: pages/objectives/sections/objectives/rows/obj-0002, text: Test}",
];

#[test]
fn a_read_key_keys_the_rows_of_a_section_without_a_channel() {
    let document = document(
        "no-channel",
        "{view: factory.Objectives, key: objective_id}",
        "",
    );
    let outcome = run(&document, STEPS);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_read_key_keys_the_rows_beside_a_channel_that_names_no_match() {
    let document = document(
        "with-channel",
        "{view: factory.Objectives, key: objective_id}",
        "        live: {channel: objectives, effect: patch_row}\n",
    );
    let outcome = run(&document, STEPS);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_live_event_matches_rows_by_the_read_key_when_live_names_no_match() {
    let document = document(
        "live-match",
        "{view: factory.Objectives, key: objective_id}",
        "        live: {channel: objectives, effect: patch_row}\n",
    );
    let outcome = run(
        &document,
        &[
            "open: objectives",
            "play: {event: factory.ObjectiveChanged, channel: objectives, with: {objective_id: obj-0002, title: Tested}}",
            "expect: {at: pages/objectives/sections/objectives/rows/obj-0002, text: Tested}",
            "expect: {at: pages/objectives/sections/objectives, rows: [obj-0001, obj-0002]}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn without_a_key_the_rows_are_still_keyed_by_id() {
    let document = document("no-key", "{view: factory.Objectives}", "");
    let outcome = run(&document, STEPS);
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
}
