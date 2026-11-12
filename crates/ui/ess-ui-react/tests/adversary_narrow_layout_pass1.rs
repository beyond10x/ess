//! Adversary pass 1 on story:a-wide-table-scrolls-inside-its-column (unit commit 5dc8a37b18).
//!
//! Each case asserts what the story's acceptance or need says against the generated project of
//! the shipped `examples/partner-portal` document. No case spawns cargo, tsc or node.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui::{Document, PageLayout};

const BREAKPOINT: &str = "@media (width < 48rem)";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn example() -> Document {
    ess_ui::load_path(&root().join("examples/partner-portal/ui.yaml"))
        .unwrap_or_else(|error| panic!("the example loads: {error}"))
}

fn files(document: &Document) -> BTreeMap<String, String> {
    ess_ui_react::render(document, &root().join("examples/partner-portal"))
        .unwrap_or_else(|error| panic!("the example generates: {error}"))
        .files
}

/// The text between the braces of the rule that opens with `head`, nested braces kept.
fn block<'a>(css: &'a str, head: &str) -> &'a str {
    let start = css
        .find(head)
        .unwrap_or_else(|| panic!("the stylesheet has no `{head}`"));
    let open = start + css[start..].find('{').expect("the rule opens");
    let mut depth = 0;
    for (at, character) in css[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &css[open + 1..open + at];
                }
            }
            _ => {}
        }
    }
    panic!("`{head}` is never closed");
}

/// The declarations of every one-line rule of `css` whose selector list names `selector`.
fn declarations_of(css: &str, selector: &str) -> Vec<String> {
    css.lines()
        .filter_map(|line| line.trim().split_once('{'))
        .filter(|(list, _)| list.split(',').any(|part| part.trim() == selector))
        .flat_map(|(_, rest)| {
            rest.trim_end_matches(['}', ' '])
                .split(';')
                .map(|part| part.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The order the runtime's `AreasLayout` draws the areas of `page` in, which is the order they
/// stack in below the renderer width (the narrow rule drops the grid template, so DOM order is
/// the visual order). Read from the generated runtime: when it orders by first appearance in the
/// grid (the rule of the unit commit), that order; otherwise the order the page passes them.
fn stacked_areas(
    core: &str,
    page_tsx: &str,
    grid: &[Vec<String>],
    areas: &[&String],
) -> Vec<String> {
    let start = core
        .find("export function AreasLayout(")
        .expect("the runtime draws an AreasLayout");
    let body = &core[start..];
    let body = &body[..body.find("\n}\n").expect("the function ends")];
    if body.contains("props.grid.flat()") && body.contains(".sort(") {
        let reading: Vec<&String> = grid.iter().flatten().collect();
        let rank = |area: &String| {
            reading
                .iter()
                .position(|cell| *cell == area)
                .unwrap_or(reading.len())
        };
        let mut ordered: Vec<&String> = areas.to_vec();
        ordered.sort_by_key(|area| rank(area));
        return ordered.into_iter().cloned().collect();
    }
    let object = &page_tsx[page_tsx.find("areas={").expect("the page passes its areas")..];
    let at = |area: &String| {
        let needle = format!("{area}: ");
        object
            .match_indices(&needle)
            .find(|(at, _)| object[..*at].ends_with(char::is_whitespace))
            .map_or(usize::MAX, |(at, _)| at)
    };
    let mut ordered: Vec<&String> = areas.to_vec();
    ordered.sort_by_key(|area| at(area));
    ordered.into_iter().cloned().collect()
}

/// Acceptance 1: "Below a fixed renderer width (`48rem`), `layout.columns` and `layout.areas`
/// stack their columns or areas into one column in declaration order, the order
/// `degrades: {no_columns: stack}` already uses". The schema names that order
/// `section_declaration` (`PageLayout.degrades`), and `PageLayout.doc` says renderers without
/// areas "fall back to a stack in section declaration order"; the TUI stacks the same page in
/// that order (`ess-ui-tui` `columns_and_areas_degrade_to_a_stack_in_section_order`).
#[test]
fn narrow_areas_stack_in_the_section_declaration_order_the_degrade_uses() {
    let schema = std::fs::read_to_string(root().join("schemas/ui/ess-ui.schema.yaml"))
        .expect("the schema reads");
    assert!(
        schema
            .lines()
            .any(|line| line.trim_start().starts_with("no_areas:")
                && line.contains("order: section_declaration")),
        "the schema names the order the no_areas stack uses"
    );

    let document = example();
    let all = files(&document);
    let core = all
        .get("src/runtime/core.tsx")
        .expect("the runtime core is generated");
    let page = &document.pages["overview"];
    let PageLayout::Areas(layout) = &page.layout else {
        panic!("the shipped overview page is laid out in areas");
    };
    let page_tsx = all
        .iter()
        .find(|(path, text)| path.starts_with("src/pages/") && text.contains("AreasLayout"))
        .map_or("", |(_, text)| text.as_str());

    let areas: Vec<&String> = layout.areas.place.keys().collect();
    let stacked: Vec<String> = stacked_areas(core, page_tsx, &layout.areas.grid, &areas)
        .iter()
        .flat_map(|area| layout.areas.place[area].sections().iter().cloned())
        .collect();
    let declared: Vec<String> = page
        .sections
        .iter()
        .map(|section| section.name.clone())
        .filter(|name| stacked.contains(name))
        .collect();
    assert_eq!(
        stacked, declared,
        "below 48rem the overview's areas stack in grid reading order, not in the section \
         declaration order acceptance 1 names (the order no_areas: stack and the TUI use)"
    );
}

/// The story's need: "A page read on a narrow window shows every section whole, one under
/// another", on "a phone-width window". Every generated app draws its regions in `.ui-shell`,
/// whose first grid track is the navigation width (`15rem`) whether or not the shell has a
/// navigation region, and the page outlet sits in the second track. Below `48rem` nothing gives
/// that track up, so on a 22.5rem phone the stacked column gets 7.5rem less the outlet padding.
#[test]
#[ignore = "backlog: narrow-shell-nav-track, story:the-shell-gives-its-navigation-track-to-the-page-on-a-narrow-window"]
fn below_the_renderer_width_the_shell_gives_its_navigation_track_to_the_page() {
    let all = files(&example());
    let shell = all
        .get("src/runtime/shell.tsx")
        .expect("the runtime shell is generated");
    assert!(
        shell.contains("className=\"ui-shell\""),
        "the shell draws its regions in .ui-shell"
    );
    let css = all
        .get("src/runtime/styles.css")
        .expect("the stylesheet is generated");
    assert!(
        declarations_of(css, ".ui-shell")
            .iter()
            .any(|part| part == "grid-template-columns: var(--ui-size-nav-width) 1fr"),
        "above the width the shell keeps a navigation track"
    );
    let narrow = block(css, BREAKPOINT);
    assert!(
        declarations_of(narrow, ".ui-shell")
            .iter()
            .any(|part| part.starts_with("grid-template-columns")),
        "below 48rem the shell still reserves var(--ui-size-nav-width) (15rem) beside the page: \
         {narrow}"
    );
}
