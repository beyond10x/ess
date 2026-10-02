//! beyond10x/ess#281: a section carries an optional `title`, its heading, so a reader can tell two
//! sections over the same view apart.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const FIXTURE: &str = "views:\n  orders.All:\n    rows:\n      \
                       - {id: o-1, total: 10}\n      - {id: o-2, total: 20}\n";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-section-title")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("orders.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(
        dir.join("ui.yaml"),
        "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures: {views: {orders.All: orders.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: overview
  sections: [{name: main, pages: [overview]}]
pages:
  overview:
    kind: static_page
    title: Overview
    sections:
      - {name: due, component: collection, title: Due this week, reads: {view: orders.All}, columns: [total]}
      - {name: overdue, component: collection, title: Overdue, reads: {view: orders.All}, columns: [total]}
      - {name: plain, component: collection, reads: {view: orders.All}, columns: [total]}
",
    )
    .expect("the document is written");
    dir.join("ui.yaml")
}

fn run(steps: &[&str]) -> Outcome {
    let document = document("shown");
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: title\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("section_title.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

#[test]
fn each_section_shows_its_own_title() {
    let outcome = run(&[
        "open: overview",
        "expect: {at: pages/overview/sections/due, text: Due this week}",
        "expect: {at: pages/overview/sections/overdue, text: Overdue}",
        "expect: {at: pages/overview/sections/due, not_text: Overdue}",
    ]);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn the_title_is_a_section_field_not_the_composites() {
    let doc = ess_ui::load_path(&document("loaded")).unwrap_or_else(|error| panic!("{error}"));
    let titles: Vec<Option<&str>> = doc.pages["overview"]
        .sections
        .iter()
        .filter(|section| section.name != "body")
        .map(|section| section.title.as_deref())
        .collect();
    assert_eq!(titles, [Some("Due this week"), Some("Overdue"), None]);
}
