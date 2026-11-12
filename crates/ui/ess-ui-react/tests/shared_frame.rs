//! A layout column or an area's place entry may draw its sections in one shared frame, a group or a
//! section may start collapsed, and a record groups its fields under headings
//! (story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed, beyond10x/ess#723
//! G24 and G25).
//!
//! The generated page holds one frame element at the column's or the area's path with the
//! sections inside it, each marked as framed; a document without the keys keeps its project.

use std::path::Path;

const HEAD: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: customer
  sections: [{name: main, pages: [customer]}]
";

const SECTIONS: &str = "    sections:
      - name: profile
        component: record
        reads: {view: shop.Customers}
        fields: [name]
        groups:
          - {name: contact, label: Contact, fields: [email, phone]}
          - {name: audit, label: Audit, collapsed: true, fields: [created_at]}
      - {name: orders, title: Open orders, component: collection, load: on_visible, reads: {view: shop.Orders}, columns: [total]}
      - {name: history, title: History, collapsed: true, component: collection, reads: {view: shop.History}, columns: [at]}
";

fn files(layout: &str) -> ess_ui_react::GeneratedFiles {
    let text = format!(
        "{HEAD}pages:\n  customer:\n    kind: detail_page\n    title: Customer\n    layout: {layout}\n{SECTIONS}"
    );
    let document = ess_ui::load_str(&text)
        .unwrap_or_else(|error| panic!("the document loads: {error}\n{text}"));
    ess_ui_react::render(&document, Path::new("."))
        .unwrap_or_else(|error| panic!("the project renders: {error}"))
}

const COLUMN: &str = "{columns: [{name: main, frame: shared, title: Customer, width: wide, sections: [profile, orders]}, {name: side, sections: [history]}]}";
const AREA: &str = "{areas: {grid: [[main, side]], place: {main: {sections: [profile, orders], frame: shared, title: Customer}, side: [history]}}}";

/// The element opened by `open` (a tag name with its `<`), up to its closing tag.
fn element<'a>(text: &'a str, name: &str) -> &'a str {
    let start = text
        .find(&format!("<{name}"))
        .unwrap_or_else(|| panic!("no <{name}> in:\n{text}"));
    let end = text[start..]
        .find(&format!("</{name}>"))
        .unwrap_or_else(|| panic!("<{name}> is never closed in:\n{text}"));
    &text[start..start + end + name.len() + 3]
}

/// The `<SectionFrame …>` tag of the page's component named `component`.
fn section_frame<'a>(page: &'a str, component: &str) -> &'a str {
    let start = page
        .find(&format!("function {component}("))
        .unwrap_or_else(|| panic!("no component {component} in:\n{page}"));
    let rest = &page[start..];
    let open = rest
        .find("<SectionFrame")
        .expect("the section draws a frame");
    // The tag ends at the first line that is only `>`, however deep the element is indented.
    let mut end = open;
    for line in rest[open..].split_inclusive('\n') {
        if line.trim() == ">" {
            return &rest[open..end];
        }
        end += line.len();
    }
    panic!("the frame's tag never ends in:\n{rest}");
}

#[test]
fn a_shared_column_is_one_frame_at_the_columns_path_holding_its_sections_in_order() {
    let files = files(COLUMN);
    let page = &files.files["src/pages/Customer.tsx"];
    let frame = element(page, "SharedFrame");
    assert!(
        frame.contains("data-ui-path=\"pages/customer/layout/columns/main\""),
        "{frame}"
    );
    assert!(frame.contains("title={\"Customer\"}"), "{frame}");
    assert!(frame.contains("width={\"wide\"}"), "{frame}");
    assert!(!frame.contains("collapsed"), "{frame}");
    let profile = frame
        .find("<CustomerProfileSection")
        .expect("the profile is inside");
    let orders = frame
        .find("<CustomerOrdersSection")
        .expect("the orders are inside");
    assert!(profile < orders, "the sections keep their order:\n{frame}");
    assert!(!frame.contains("<CustomerHistorySection"), "{frame}");
    assert_eq!(page.matches("<SharedFrame").count(), 1, "{page}");
    assert!(
        page.contains("<LayoutColumn"),
        "the unshared column stays one:\n{page}"
    );
    for component in ["CustomerProfileSection", "CustomerOrdersSection"] {
        let tag = section_frame(page, component);
        assert!(
            tag.contains("framed={true}"),
            "{component} is not marked framed:\n{tag}"
        );
    }
    let history = section_frame(page, "CustomerHistorySection");
    assert!(!history.contains("framed"), "{history}");
}

#[test]
fn a_shared_area_is_one_frame_at_the_areas_path() {
    let files = files(AREA);
    let page = &files.files["src/pages/Customer.tsx"];
    let frame = element(page, "SharedFrame");
    assert!(
        frame.contains("data-ui-path=\"pages/customer/layout/areas/place/main\""),
        "{frame}"
    );
    assert!(frame.contains("title={\"Customer\"}"), "{frame}");
    assert!(
        !frame.contains("width="),
        "an area has no column width:\n{frame}"
    );
    assert!(frame.contains("<CustomerProfileSection") && frame.contains("<CustomerOrdersSection"));
    assert!(section_frame(page, "CustomerProfileSection").contains("framed={true}"));
    assert!(!section_frame(page, "CustomerHistorySection").contains("framed"));
}

#[test]
fn a_collapsed_shared_frame_starts_closed() {
    let files = files(
        "{columns: [{name: main, frame: shared, title: Customer, collapsed: true, sections: [profile, orders, history]}]}",
    );
    let page = &files.files["src/pages/Customer.tsx"];
    let frame = element(page, "SharedFrame");
    assert!(frame.contains("collapsed={true}"), "{frame}");
}

#[test]
fn a_collapsed_section_and_a_collapsed_record_group_start_closed() {
    let files = files(COLUMN);
    let page = &files.files["src/pages/Customer.tsx"];
    let history = section_frame(page, "CustomerHistorySection");
    assert!(history.contains("collapsed={true}"), "{history}");
    assert!(history.contains("title={\"History\"}"), "{history}");
    assert!(!section_frame(page, "CustomerOrdersSection").contains("collapsed"));
    let record = page
        .find("<RecordView")
        .map(|start| &page[start..])
        .expect("the profile is a record");
    let groups = &record[record.find("groups={").expect("the record has groups")..];
    let contact = &groups[groups.find("\"contact\"").expect("the contact group")..];
    let audit = &groups[groups.find("\"audit\"").expect("the audit group")..];
    assert!(
        groups.find("\"contact\"") < groups.find("\"audit\""),
        "the groups keep their order"
    );
    assert!(contact.contains("label: \"Contact\""), "{contact}");
    assert!(
        audit.contains("label: \"Audit\"") && audit.contains("collapsed: true"),
        "{audit}"
    );
    let contact_only = &contact[..contact.find("\"audit\"").unwrap_or(contact.len())];
    assert!(!contact_only.contains("collapsed"), "{contact_only}");
}

#[test]
fn a_collapsed_form_group_names_collapsed_and_keeps_its_fields() {
    let text = format!(
        "{HEAD}pages:\n  customer:\n    kind: detail_page\n    title: Customer\n    sections:
      - name: settings
        component: form
        does: shop.Save
        groups:
          - {{name: basics, label: Basics, fields: [name]}}
          - {{name: advanced, label: Advanced, collapsed: true, fields: [limit]}}
"
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let files = ess_ui_react::render(&document, Path::new(".")).expect("the project renders");
    let page = &files.files["src/pages/Customer.tsx"];
    let advanced = &page[page.find("\"advanced\"").expect("the advanced group")..];
    assert!(advanced.contains("collapsed: true"), "{advanced}");
    assert!(
        advanced.contains("\"limit\""),
        "the collapsed group still lists its fields:\n{advanced}"
    );
    let basics = &page
        [page.find("\"basics\"").expect("the basics group")..page.find("\"advanced\"").unwrap()];
    assert!(!basics.contains("collapsed"), "{basics}");
}

#[test]
fn the_runtime_draws_a_collapsed_heading_as_a_button_over_a_body_that_stays_mounted() {
    let files = files(COLUMN);
    let core = &files.files["src/runtime/core.tsx"];
    let shared = &core[core
        .find("export function SharedFrame")
        .expect("the runtime defines SharedFrame")..];
    let shared = &shared[..shared.find("\n}\n").unwrap_or(shared.len())];
    assert!(shared.contains("aria-expanded"), "{shared}");
    assert!(
        shared.contains("<button type=\"button\""),
        "a button opens it on click and on Enter:\n{shared}"
    );
    assert!(
        shared.contains("hidden="),
        "a closed body is hidden, not unmounted:\n{shared}"
    );
    let section = &core[core.find("export function SectionFrame").unwrap()..];
    let section = &section[..section.find("\n}\n").unwrap_or(section.len())];
    assert!(
        section.contains("framed?: boolean") && section.contains("collapsed?: boolean"),
        "{section}"
    );
    assert!(section.contains("aria-expanded"), "{section}");
}

#[test]
fn a_document_without_the_keys_keeps_its_project() {
    let plain =
        "{columns: [{name: main, sections: [profile, orders]}, {name: side, sections: [history]}]}";
    let text = format!(
        "{HEAD}pages:\n  customer:\n    kind: detail_page\n    title: Customer\n    layout: {plain}
    sections:
      - {{name: profile, component: record, reads: {{view: shop.Customers}}, fields: [name]}}
      - {{name: orders, title: Open orders, component: collection, reads: {{view: shop.Orders}}, columns: [total]}}
      - {{name: history, title: History, component: collection, reads: {{view: shop.History}}, columns: [at]}}
"
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let files = ess_ui_react::render(&document, Path::new(".")).expect("the project renders");
    for (path, text) in &files.files {
        let words: &[&str] = if path.starts_with("src/pages/") {
            &["SharedFrame", "framed", "collapsed", "groups"]
        } else {
            &[
                "SharedFrame",
                ".ui-collapse ",
                "ui-collapse-body",
                "ui-frame",
                "ui-record-group",
                "GroupBox",
                "RecordGroupSpec",
            ]
        };
        for word in words {
            assert!(
                !text.contains(word),
                "{path} names `{word}` in a document without the keys"
            );
        }
    }
}
