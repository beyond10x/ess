//! beyond10x/ess#281: a section's `title` reaches the generated section frame, which renders it as
//! the section's heading.

use std::path::Path;

const DOCUMENT: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
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
      - {name: plain, component: collection, reads: {view: orders.All}, columns: [total]}
";

fn files() -> ess_ui_react::GeneratedFiles {
    let document =
        ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::render(&document, Path::new("."))
        .unwrap_or_else(|error| panic!("the project renders: {error}"))
}

#[test]
fn the_section_frame_is_given_the_title() {
    let files = files();
    let page = &files.files["src/pages/Overview.tsx"];
    let frames: Vec<&str> = page
        .match_indices("<SectionFrame")
        .map(|(start, _)| {
            let end = page[start..]
                .find(">\n")
                .map_or(page.len(), |end| start + end);
            &page[start..end]
        })
        .collect();
    assert!(
        frames
            .iter()
            .any(|frame| frame.contains("title=\"Due this week\"")
                || frame.contains("title={\"Due this week\"}")),
        "no section frame carries the title:\n{page}"
    );
    assert_eq!(
        page.matches("Due this week").count(),
        1,
        "only the titled section carries a title:\n{page}"
    );
}

#[test]
fn the_section_frame_renders_its_title_as_a_heading() {
    let files = files();
    let core = &files.files["src/runtime/core.tsx"];
    let start = core
        .find("export function SectionFrame")
        .expect("the runtime defines SectionFrame");
    let frame = &core[start..];
    let frame = &frame[..frame.find("\n}\n").unwrap_or(frame.len())];
    assert!(
        frame.contains("title?: string"),
        "SectionFrame takes no title:\n{frame}"
    );
    assert!(
        frame.contains("ui-section-title") && frame.contains("{props.title}"),
        "SectionFrame does not render its title as a heading:\n{frame}"
    );
}
