//! A page laid out in `layout.columns` or `layout.areas` stacks into one column below a fixed
//! renderer width, and a wide table scrolls inside its column instead of widening the page
//! (story:a-wide-table-scrolls-inside-its-column, beyond10x/ess#723 D13).
//!
//! The width is the renderer's, not a key of the document: the stylesheet holds one `@media`
//! rule, and a document's page files do not change with it.

use std::collections::BTreeMap;
use std::path::Path;

const BREAKPOINT: &str = "@media (width < 48rem)";

const DOC: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: customer
  sections: [{name: main, pages: [customer, account, overview]}]
pages:
  customer:
    kind: detail_page
    title: Customer
    layout: {columns: [{name: main, sections: [wide]}, {name: side, width: narrow, sections: [notes]}]}
    sections:
      - name: wide
        component: collection
        reads: {view: shop.Orders}
        columns: [c1, c2, c3, c4, c5, c6, c7, c8, c9, c10]
      - {name: notes, component: collection, reads: {view: shop.Notes}, columns: [text]}
  account:
    kind: detail_page
    title: Account
    layout: {areas: {grid: [[main, side]], place: {main: [wide], side: [notes]}}}
    sections:
      - name: wide
        component: collection
        reads: {view: shop.Orders}
        columns: [c1, c2, c3, c4, c5, c6, c7, c8, c9, c10]
      - {name: notes, component: collection, reads: {view: shop.Notes}, columns: [text]}
  overview:
    kind: detail_page
    title: Overview
    layout: {areas: {grid: [[zeta, alpha], [mid, mid]], place: {alpha: [notes], mid: [wide], zeta: [tail]}}}
    sections:
      - name: wide
        component: collection
        reads: {view: shop.Orders}
        columns: [c1]
      - {name: notes, component: collection, reads: {view: shop.Notes}, columns: [text]}
      - {name: tail, component: collection, reads: {view: shop.Tail}, columns: [at]}
";

fn files() -> BTreeMap<String, String> {
    let document =
        ess_ui::load_str(DOC).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::render(&document, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("the document generates: {error}"))
        .files
}

fn styles(files: &BTreeMap<String, String>) -> &str {
    files
        .get("src/runtime/styles.css")
        .unwrap_or_else(|| panic!("no styles.css in {:?}", files.keys()))
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

/// `css` without the rule that opens with `head`.
fn without(css: &str, head: &str) -> String {
    let inner = block(css, head);
    css.replacen(inner, "", 1)
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

fn declares(css: &str, selector: &str, declaration: &str) {
    let found = declarations_of(css, selector);
    assert!(
        found.iter().any(|part| part == declaration),
        "`{selector}` lacks `{declaration}`: {found:?}"
    );
}

#[test]
fn columns_and_areas_stack_below_the_renderer_width() {
    let all = files();
    let css = styles(&all);
    assert_eq!(
        css.matches("@media").count(),
        1,
        "one renderer width, one media rule"
    );
    let narrow = block(css, BREAKPOINT);

    declares(narrow, ".ui-layout-columns", "flex-direction: column");
    declares(narrow, ".ui-layout-columns", "align-items: stretch");
    for column in [".ui-column", ".ui-column-narrow", ".ui-column-wide"] {
        declares(narrow, column, "flex: none");
    }

    declares(
        narrow,
        ".ui-layout-areas",
        "grid-template-columns: minmax(0, 1fr) !important",
    );
    declares(
        narrow,
        ".ui-layout-areas",
        "grid-template-areas: none !important",
    );
    declares(narrow, ".ui-area", "grid-area: auto !important");

    let wide = without(css, BREAKPOINT);
    declares(&wide, ".ui-layout-columns", "display: flex");
    assert!(
        !declarations_of(&wide, ".ui-layout-columns")
            .iter()
            .any(|part| part.starts_with("flex-direction")),
        "above the width the columns lay out side by side as before"
    );
    assert!(
        !declarations_of(&wide, ".ui-layout-areas")
            .iter()
            .any(|part| part.contains("!important")),
        "above the width the areas keep the grid the page declares"
    );
}

#[test]
fn a_wide_table_scrolls_inside_its_column() {
    let all = files();
    let css = styles(&all);
    declares(css, ".ui-table-frame", "overflow-x: auto");
    declares(css, ".ui-table-frame", "max-width: 100%");
    for shrinking in [".ui-column", ".ui-area", ".ui-section"] {
        declares(css, shrinking, "min-width: 0");
    }

    let areas = all
        .get("src/runtime/core.tsx")
        .expect("the runtime core is generated");
    assert!(
        areas.contains("minmax(0, 1fr)"),
        "an area's grid track may shrink below its content"
    );

    let collection = all
        .get("src/runtime/composites/collection.tsx")
        .expect("the collection composite is generated");
    assert!(
        collection.contains("<div className=\"ui-table-frame\">"),
        "the table sits in the scrolling frame"
    );
    for (page, layout) in [
        ("Customer.tsx", "ColumnsLayout"),
        ("Account.tsx", "AreasLayout"),
    ] {
        let (path, text) = all
            .iter()
            .find(|(path, text)| path.contains(page) && text.contains(layout))
            .unwrap_or_else(|| panic!("no {page} page draws a {layout}: {:?}", all.keys()));
        for column in 1..=10 {
            assert!(
                text.contains(&format!("c{column}")),
                "{path} lacks column c{column}"
            );
        }
    }
}

#[test]
fn areas_stack_in_the_section_declaration_order() {
    let all = files();
    let page = all
        .get("src/pages/Overview.tsx")
        .expect("the overview page is generated");
    let object = &page[page
        .find("areas={")
        .unwrap_or_else(|| panic!("the page passes its areas:\n{page}"))..];
    // The grid reads zeta, alpha, mid; the names sort alpha, mid, zeta; the sections are
    // declared wide (placed in mid), notes (alpha), tail (zeta).
    let key = |area: &str| {
        object
            .match_indices(&format!("{area}: "))
            .find(|(at, _)| object[..*at].ends_with(char::is_whitespace))
            .map_or_else(|| panic!("no `{area}` key in:\n{object}"), |(at, _)| at)
    };
    let mut keys: Vec<&str> = vec!["alpha", "mid", "zeta"];
    keys.sort_by_key(|area| key(area));
    assert_eq!(
        keys,
        ["mid", "alpha", "zeta"],
        "the page passes its areas in section declaration order:\n{object}"
    );

    // The order is passed as a list too: `Object.entries` would put an integer-like area name
    // (`1`, which the name pattern admits) first whatever order the keys are written in.
    assert!(
        page.contains(r#"order={["mid", "alpha", "zeta"]}"#),
        "the page passes its stacking order as a list:\n{object}"
    );
    let core = all
        .get("src/runtime/core.tsx")
        .expect("the runtime core is generated");
    assert!(
        core.contains("props.order.map(") && !core.contains("Object.entries(props.areas)"),
        "the runtime draws the areas in the order list the page passes"
    );
}
