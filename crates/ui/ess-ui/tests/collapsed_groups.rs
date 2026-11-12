//! A record's groups, `collapsed` on a form group, a record group and a section, and a layout
//! column or area that frames its sections
//! (story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed, beyond10x/ess#723
//! G24 and G25): what the loader reads, where it reports a refusal, and which nodes it walks.

use ess_ui::{load_str, AreaPlace, Composite, NodeRef, PageLayout};

fn document(page: &str) -> String {
    format!(
        "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: customer
  sections: [{{name: all, pages: [customer]}}]
pages:
  customer:
    kind: detail_page
    title: Customer
{page}"
    )
}

const SECTIONS: &str = "    sections:
      - {name: profile, title: Profile, collapsed: true, component: record, reads: {view: shop.Customers}, fields: [name], groups: [{name: contact, label: Contact, fields: [email]}, {name: audit, label: Audit, collapsed: true, fields: [created_by]}]}
      - {name: orders, component: collection, reads: {view: shop.Orders}, columns: [total]}
";

#[test]
fn a_record_reads_its_groups_and_a_section_its_collapsed_flag() {
    let loaded = load_str(&document(SECTIONS)).unwrap_or_else(|error| panic!("{error}"));
    let page = &loaded.pages["customer"];
    let section = |name: &str| {
        page.sections
            .iter()
            .find(|section| section.name == name)
            .expect("the section")
    };
    let profile = section("profile");
    assert_eq!(profile.collapsed, Some(true));
    assert_eq!(section("orders").collapsed, None);
    let ess_ui::Body::Composite(Composite::Record(record)) = &profile.body else {
        panic!("the profile is a record");
    };
    let names: Vec<_> = record
        .groups
        .iter()
        .map(|group| group.name.as_str())
        .collect();
    assert_eq!(names, ["contact", "audit"]);
    assert_eq!(record.groups[0].label.as_deref(), Some("Contact"));
    assert_eq!(record.groups[0].collapsed, None);
    assert_eq!(record.groups[1].collapsed, Some(true));
    assert_eq!(record.groups[1].fields[0].field, "created_by");
}

#[test]
fn a_form_group_reads_collapsed() {
    let text = document(
        "    sections:
      - name: settings
        component: form
        does: shop.Save
        groups: [{name: advanced, label: Advanced, collapsed: true, fields: [limit]}]
",
    );
    let loaded = load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let ess_ui::Body::Composite(Composite::Form(form)) = &loaded.pages["customer"]
        .sections
        .iter()
        .find(|section| section.name == "settings")
        .expect("settings")
        .body
    else {
        panic!("a form");
    };
    assert_eq!(form.groups[0].collapsed, Some(true));
}

#[test]
fn a_column_and_an_area_read_how_they_frame_their_sections() {
    let columns = document(&format!(
        "    layout: {{columns: [{{name: main, frame: shared, title: Customer, collapsed: true, sections: [profile, orders]}}]}}\n{SECTIONS}"
    ));
    let loaded = load_str(&columns).unwrap_or_else(|error| panic!("{error}"));
    let PageLayout::Columns(layout) = &loaded.pages["customer"].layout else {
        panic!("columns");
    };
    let column = &layout.columns[0];
    assert!(column.is_shared());
    assert_eq!(column.title.as_deref(), Some("Customer"));
    assert_eq!(column.collapsed, Some(true));

    let areas = document(&format!(
        "    layout: {{areas: {{grid: [[a, b]], place: {{a: {{sections: [profile], frame: shared, title: Customer}}, b: [orders]}}}}}}\n{SECTIONS}"
    ));
    let loaded = load_str(&areas).unwrap_or_else(|error| panic!("{error}"));
    let PageLayout::Areas(layout) = &loaded.pages["customer"].layout else {
        panic!("areas");
    };
    let AreaPlace::Framed(framed) = &layout.areas.place["a"] else {
        panic!("the record form");
    };
    assert!(framed.is_shared());
    assert_eq!(framed.title.as_deref(), Some("Customer"));
    assert_eq!(layout.areas.place["a"].sections(), ["profile"]);
    assert_eq!(
        layout.areas.place["b"],
        AreaPlace::Sections(vec!["orders".into()])
    );
    assert_eq!(layout.areas.place["b"].sections(), ["orders"]);
}

#[test]
fn the_walk_reaches_a_record_group_and_an_area_by_path() {
    let text = document(&format!(
        "    layout: {{areas: {{grid: [[a, b]], place: {{a: {{sections: [profile], frame: shared, title: Customer}}, b: [orders]}}}}}}\n{SECTIONS}"
    ));
    let loaded = load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let nodes = loaded.nodes();
    let at = |path: &str| {
        nodes
            .iter()
            .find(|located| located.path.to_string() == path)
            .unwrap_or_else(|| panic!("no node at `{path}`"))
    };
    assert!(matches!(
        at("pages/customer/sections/profile/groups/audit").node,
        NodeRef::RecordGroup(group) if group.collapsed == Some(true)
    ));
    assert!(matches!(
        at("pages/customer/sections/profile/groups/audit/fields/created_by").node,
        NodeRef::Field(_)
    ));
    assert!(matches!(
        at("pages/customer/layout/areas/place/a").node,
        NodeRef::LayoutArea(AreaPlace::Framed(_))
    ));
    assert!(matches!(
        at("pages/customer/layout/areas/place/b").node,
        NodeRef::LayoutArea(AreaPlace::Sections(_))
    ));
}

#[test]
fn a_record_group_takes_no_save_key_and_is_refused_at_its_own_path() {
    for key in ["save", "does", "actions"] {
        let value = if key == "actions" { "[]" } else { "x" };
        let text = document(&format!(
            "    sections:
      - {{name: profile, component: record, reads: {{view: shop.Customers}}, groups: [{{name: first, fields: [name]}}, {{name: contact, label: Contact, {key}: {value}, fields: [email]}}]}}
"
        ));
        let error = load_str(&text).expect_err("a record group has no save key");
        assert_eq!(
            error.path().to_string(),
            "pages/customer/sections/profile/groups/contact",
            "{key}: {error}"
        );
        assert!(
            error.message().contains(&format!("`{key}`")),
            "{key}: {error}"
        );
    }
}

#[test]
fn a_form_group_still_takes_its_save_keys() {
    let text = document(
        "    sections:
      - name: settings
        component: form
        does: shop.Save
        groups: [{name: basics, label: Basics, save: on_change, does: shop.SaveBasics, fields: [name]}]
",
    );
    load_str(&text).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_refused_layout_key_is_named() {
    let text = document(&format!(
        "    layout: {{columns: [{{name: main, framed: shared, sections: [profile, orders]}}]}}\n{SECTIONS}"
    ));
    let error = load_str(&text).expect_err("`framed` is no key");
    assert!(error.message().contains("`framed`"), "{error}");
    let text = document(&format!(
        "    layout: {{areas: {{grid: [[a]], place: {{a: {{sections: [profile, orders], frames: shared}}}}}}}}\n{SECTIONS}"
    ));
    let error = load_str(&text).expect_err("`frames` is no key");
    assert!(error.message().contains("`frames`"), "{error}");
}
