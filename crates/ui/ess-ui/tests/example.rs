//! The partner-portal example loads, uses every construct, and addresses every node by a path
//! that survives the insertion of a sibling.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_ui::{
    ActionConfirm, Body, Composite, Document, Located, NavPages, NodeRef, PageLayout, Primitive,
};
use serde_yaml::{Mapping, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn example_file() -> PathBuf {
    root().join("examples/partner-portal/ui.yaml")
}

fn example_value() -> Value {
    let text = std::fs::read_to_string(example_file()).expect("the example reads");
    serde_yaml::from_str(&text).expect("the example is YAML")
}

fn load_value(value: &Value) -> Result<Document, ess_ui::LoadError> {
    ess_ui::load_str(&serde_yaml::to_string(value).expect("a value writes"))
}

fn example() -> Document {
    ess_ui::load_path(&example_file()).unwrap_or_else(|error| panic!("{error}"))
}

fn schema() -> Value {
    serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML")
}

fn paths(document: &Document) -> BTreeSet<String> {
    document
        .nodes()
        .iter()
        .map(|located| located.path.to_string())
        .collect()
}

#[test]
fn the_partner_portal_example_loads() {
    let document = example();
    assert_eq!(document.format, "ess-ui/1");
    assert_eq!(document.pages.len(), 15);
    let summary = ess_ui::check(&example_file()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(summary.pages, 15);
    assert_eq!(summary.nodes, document.nodes().len());
    assert!(summary.nodes > 200, "{summary}");
}

#[test]
fn every_node_path_example_in_the_schema_names_a_node_of_the_example() {
    let schema = schema();
    let examples = schema["constructs"]["NodePath"]["examples"]
        .as_sequence()
        .expect("NodePath lists examples");
    let found = paths(&example());
    let mut checked = 0;
    for example in examples {
        let example = example.as_str().expect("a path example is a string");
        if example.contains("/rows/") {
            continue; // rows are runtime data, addressed by row key
        }
        assert!(found.contains(example), "no node at `{example}`");
        checked += 1;
    }
    assert!(checked >= 6, "only {checked} path examples were checked");
}

#[test]
fn widget_instances_carry_their_expanded_body() {
    let document = example();
    let found = paths(&document);
    for path in [
        "pages/partners.list/sections/list/item/card/body/logo",
        "pages/partners.list/sections/list/item/card/body/tier/body/badge",
        "pages/partners.list/sections/list/item/card/body/revenue/body/amount",
        "pages/partners.detail/sections/summary/item/revenue/body/amount",
    ] {
        assert!(found.contains(path), "no node at `{path}`");
    }
    let card = located(
        &document,
        "pages/partners.list/sections/list/item/card/body/title",
    );
    let NodeRef::Node(node) = card.node else {
        panic!("not a node: {card:?}")
    };
    let Body::Primitive(Primitive::Text(text)) = &node.body else {
        panic!("not a text: {node:?}")
    };
    assert_eq!(
        text.text.as_ref().map(|expr| expr.0.as_str()),
        Some("row.name")
    );
    let badge = located(
        &document,
        "pages/partners.detail/sections/deals/item/stage/body/badge",
    );
    let NodeRef::Node(node) = badge.node else {
        panic!("not a node: {badge:?}")
    };
    let Body::Primitive(Primitive::Badge(badge)) = &node.body else {
        panic!("not a badge: {node:?}")
    };
    let tone_by = badge.tone_by.as_ref().expect("the badge is toned by value");
    assert_eq!(tone_by.value.0, "row.stage");
    assert_eq!(tone_by.tones.as_deref(), Some("deal_stage"));
    assert_eq!(tone_by.map["won"], Value::from("success"));
}

fn located<'a>(document: &'a Document, path: &str) -> Located<'a> {
    document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at `{path}`"))
}

/// Which constructs of the schema the document uses, by construct name.
#[allow(clippy::too_many_lines)] // one arm per construct: splitting it would hide which are counted
fn constructs_used(document: &Document) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    let mut mark = |name: &str| {
        used.insert(name.to_owned());
    };
    if document.format == ess_ui::FORMAT {
        mark("Document");
    }
    if !document.navigation.sections.is_empty() {
        mark("Navigation");
    }
    if !document.nodes().is_empty() {
        mark("NodePath");
    }
    if !document.types.is_empty() {
        mark("Type");
    }
    if !document.page_kinds.is_empty() {
        mark("PageKind");
    }
    if document.fixtures.is_some() {
        mark("FixtureIndex");
    }
    // A theme's overrides are written in the shape of `tokens:`.
    if !document.tokens.is_empty() || document.themes.values().any(|theme| !theme.is_empty()) {
        mark("Tokens");
    }
    if !document.themes.is_empty() {
        mark("Theme");
    }
    if document.theme.is_some() {
        mark("ThemeChoice");
    }
    if !document.tone_maps.is_empty() {
        mark("ToneMap");
    }
    if document.placement_profile == ess_ui::PlacementProfile::Hybrid
        && document.pages.values().any(|page| page.profile.is_some())
    {
        mark("PlacementProfile");
    }
    if document
        .navigation
        .sections
        .iter()
        .any(|section| matches!(section.pages, NavPages::Dynamic(_)))
    {
        mark("DynamicNavEntries");
    }
    let layouts: BTreeSet<&str> = document
        .pages
        .values()
        .map(|page| match page.layout {
            PageLayout::Stack(_) => "stack",
            PageLayout::Columns(_) => "columns",
            PageLayout::Areas(_) => "areas",
        })
        .collect();
    if layouts.len() == 3 {
        mark("PageLayout");
    }
    let mut effects = BTreeSet::new();
    let mut placeholder = false;
    for located in document.nodes() {
        match located.node {
            NodeRef::Shell(shell) => {
                mark("Shell");
                if shell.preload.is_some() {
                    mark("Preload");
                }
            }
            NodeRef::Region(_) => mark("Region"),
            NodeRef::Guard(_) => mark("Guard"),
            NodeRef::State(state) => {
                mark("State");
                mark("StateClass");
                if state.store.is_some() {
                    mark("Store");
                }
            }
            NodeRef::NavSection(_) => mark("NavSection"),
            NodeRef::PageKind(_) | NodeRef::LayoutColumn(_) => {}
            NodeRef::Widget(_) => mark("Widget"),
            NodeRef::Channel(_) => mark("Channel"),
            NodeRef::Page(page) => {
                mark("Page");
                if page.nav.is_some() {
                    mark("NavEntry");
                }
            }
            NodeRef::Header(_) => mark("header"),
            NodeRef::Section(section) => {
                mark("Section");
                if section.states.is_some() {
                    mark("SectionStates");
                }
                if section.live.is_some() {
                    mark("Live");
                }
                placeholder |= body_reads_placeholder(&section.body);
                if !section.common.degrades.is_empty() {
                    mark("Degrades");
                }
                body(&section.body, &mut mark);
            }
            NodeRef::Overlay(overlay) => {
                mark("overlay");
                body(&overlay.body, &mut mark);
            }
            NodeRef::Node(node) => {
                mark("Node");
                if !node.common.degrades.is_empty() {
                    mark("Degrades");
                }
                body(&node.body, &mut mark);
            }
            NodeRef::Field(_) => mark("Field"),
            NodeRef::Action(action) => {
                mark("Action");
                for (effect, present) in [
                    ("does", action.does.is_some()),
                    ("opens", action.opens.is_some()),
                    ("navigate", action.navigate.is_some()),
                    ("export", action.export.is_some()),
                    ("upload", action.upload.is_some()),
                    ("copy", action.copy.is_some()),
                    ("sets", !action.sets.is_empty()),
                ] {
                    if present {
                        effects.insert(effect);
                    }
                }
                if matches!(action.confirm, Some(ActionConfirm::Inline(_))) {
                    effects.insert("confirm");
                }
            }
            NodeRef::Tab(_) => mark("Tab"),
            NodeRef::FormGroup(_) => mark("FormGroup"),
        }
    }
    assert_eq!(
        effects.len(),
        8,
        "every action effect and an inline confirm are used: {effects:?}"
    );
    assert!(placeholder, "an unbound placeholder read is used");
    used
}

fn reads_of(composite: &Composite) -> bool {
    match composite {
        Composite::Collection(collection) => collection.reads.is_some(),
        Composite::Record(record) => record.reads.is_some(),
        Composite::Form(form) => form.loads.is_some(),
        Composite::Choice(choice) => choice.reads.is_some(),
        Composite::Metric(metric) => metric.reads.is_some(),
        Composite::Chart(_)
        | Composite::Board(_)
        | Composite::GraphEditor(_)
        | Composite::References(_) => true,
        Composite::FilterBar(_) | Composite::Confirm(_) | Composite::RichText(_) => false,
    }
}

fn body_reads_placeholder(body: &Body) -> bool {
    matches!(body, Body::Composite(Composite::Chart(chart)) if chart.reads.placeholder.is_some())
}

fn body(body: &Body, mark: &mut impl FnMut(&str)) {
    match body {
        Body::Composite(composite) => {
            mark("Composite");
            if reads_of(composite) {
                mark("Reads");
            }
            mark(match composite {
                Composite::Collection(_) => "collection",
                Composite::Record(_) => "record",
                Composite::Form(_) => "form",
                Composite::Choice(_) => "choice",
                Composite::FilterBar(_) => "filter_bar",
                Composite::Confirm(_) => "confirm",
                Composite::Metric(_) => "metric",
                Composite::Chart(_) => "chart",
                Composite::Board(_) => "board",
                Composite::GraphEditor(_) => "graph_editor",
                Composite::RichText(_) => "rich_text",
                Composite::References(_) => "references",
            });
        }
        Body::Widget(_) => mark("WidgetInstance"),
        Body::Primitive(primitive) => {
            mark("Primitive");
            mark(match primitive {
                Primitive::Text(_) => "text",
                Primitive::Badge(_) => "badge",
                Primitive::Icon(_) => "icon",
                Primitive::Button(_) => "button",
                Primitive::Link(_) => "link",
                Primitive::Input(_) => "input",
                Primitive::Toggle(_) => "toggle",
                Primitive::Image(_) => "image",
                Primitive::Divider(_) => "divider",
            });
        }
    }
}

#[test]
fn the_example_uses_every_construct_of_the_schema() {
    let schema = schema();
    let declared: BTreeSet<String> = schema["constructs"]
        .as_mapping()
        .expect("constructs is a map")
        .keys()
        .map(|key| key.as_str().expect("a construct name").to_owned())
        .collect();
    let used = constructs_used(&example());
    let unused: Vec<&String> = declared.difference(&used).collect();
    assert!(
        unused.is_empty(),
        "constructs the example never uses: {unused:?}"
    );
}

/// Inserts `entry` at the front of the sequence reached by `keys` (a list entry is chosen by name).
fn insert_first(document: &mut Value, keys: &[&str], entry: Value) {
    let mut at = document;
    for key in keys {
        at = match at {
            Value::Mapping(mapping) => mapping.get_mut(*key).expect("the key exists"),
            Value::Sequence(entries) => entries
                .iter_mut()
                .find(|entry| entry["name"].as_str() == Some(key))
                .expect("the named entry exists"),
            other => panic!("cannot descend into {other:?}"),
        };
    }
    at.as_sequence_mut()
        .expect("the target is a list")
        .insert(0, entry);
}

fn entry(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).expect("an entry is YAML")
}

#[test]
fn paths_are_stable_when_a_sibling_is_inserted() {
    let before = example();
    let described: BTreeMap<String, String> = before
        .nodes()
        .iter()
        .map(|located| (located.path.to_string(), format!("{:?}", located.node)))
        .collect();

    let mut changed = example_value();
    let inserted = [
        (
            vec!["pages", "partners.list", "sections"],
            "{name: banner, component: record, reads: {view: partners.Banner}}",
            "pages/partners.list/sections/banner",
        ),
        (
            vec!["pages", "partners.list", "sections", "list", "columns"],
            "{field: region, as: badge}",
            "pages/partners.list/sections/list/columns/region",
        ),
        (
            vec!["pages", "overview", "sections", "kpis", "item"],
            "{name: churn, component: metric, from: channel.metrics.open_deals}",
            "pages/overview/sections/kpis/item/churn",
        ),
        (
            vec!["navigation", "sections"],
            "{name: first, label: First, pages: []}",
            "navigation/sections/first",
        ),
        (
            vec!["widgets", "partner_card", "body"],
            "{name: rule_top, primitive: divider}",
            "widgets/partner_card/body/rule_top",
        ),
    ];
    for (keys, yaml, _) in &inserted {
        insert_first(&mut changed, keys, entry(yaml));
    }
    let after = load_value(&changed).unwrap_or_else(|error| panic!("{error}"));
    let after_paths = paths(&after);
    for (_, _, path) in &inserted {
        assert!(
            after_paths.contains(*path),
            "the inserted node is at `{path}`"
        );
    }
    let after_described: BTreeMap<String, String> = after
        .nodes()
        .iter()
        .map(|located| (located.path.to_string(), format!("{:?}", located.node)))
        .collect();
    let mut compared = 0;
    for (path, description) in &described {
        let now = after_described
            .get(path)
            .unwrap_or_else(|| panic!("`{path}` vanished after inserting siblings"));
        // An ancestor of an inserted node holds it, so it rightly compares different; so does
        // every use of the widget whose body gained a node.
        let holds_an_insertion = inserted
            .iter()
            .any(|(_, _, inserted)| inserted.starts_with(&format!("{path}/")))
            || path.ends_with("/item/card");
        if !holds_an_insertion {
            assert_eq!(now, description, "`{path}` names a different node now");
            compared += 1;
        }
    }
    assert!(compared > 150, "only {compared} nodes were compared");
}

fn refused(document: &Value) -> ess_ui::LoadError {
    match load_value(document) {
        Ok(_) => panic!("the document was accepted"),
        Err(error) => error,
    }
}

fn set(document: &mut Value, keys: &[&str], key: &str, value: Value) {
    let mut at = document;
    for step in keys {
        at = match at {
            Value::Mapping(mapping) => mapping.get_mut(*step).expect("the key exists"),
            Value::Sequence(entries) => entries
                .iter_mut()
                .find(|entry| entry["name"].as_str() == Some(step))
                .expect("the named entry exists"),
            other => panic!("cannot descend into {other:?}"),
        };
    }
    let mapping: &mut Mapping = at.as_mapping_mut().expect("the target is a map");
    mapping.insert(Value::from(key), value);
}

#[test]
fn an_unknown_property_on_a_composite_is_refused_naming_its_path() {
    let cases: [(&[&str], &str); 5] = [
        (
            &["pages", "partners.list", "sections", "list"],
            "pages/partners.list/sections/list",
        ),
        (
            &["pages", "overview", "sections", "kpis", "item", "revenue"],
            "pages/overview/sections/kpis/item/revenue",
        ),
        (
            &["pages", "tickets.detail", "overlays", "close"],
            "pages/tickets.detail/overlays/close",
        ),
        (
            &[
                "pages", "overview", "sections", "board", "widgets", "win_rate",
            ],
            "pages/overview/sections/board/widgets/win_rate",
        ),
        (
            &["widgets", "partner_card", "body", "title"],
            "widgets/partner_card/body/title",
        ),
    ];
    for (keys, path) in cases {
        let mut document = example_value();
        set(&mut document, keys, "bogus_property", Value::from(1));
        let error = refused(&document);
        assert_eq!(error.path().to_string(), path, "{error}");
        assert!(
            error.message().contains("bogus_property"),
            "the refusal names the property: {error}"
        );
    }
}

fn names_under(document: &Document, prefix: &str) -> Vec<String> {
    let depth = prefix.split('/').count() + 1;
    document
        .nodes()
        .iter()
        .map(|located| located.path.to_string())
        .filter(|path| path.starts_with(&format!("{prefix}/")) && path.split('/').count() == depth)
        .map(|path| path.rsplit('/').next().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn the_page_kind_chain_is_merged_into_its_pages_by_name_in_order() {
    let document = example();
    assert_eq!(
        names_under(&document, "pages/partners.list/sections"),
        ["filters", "list"]
    );
    assert_eq!(
        names_under(&document, "pages/partners.list/sections/filters/choices"),
        ["tags", "tier"],
        "inherited entries first, then the page's own"
    );
    assert_eq!(
        names_under(&document, "pages/users.list/sections/filters/choices"),
        ["roles"],
        "`{{name: tags, remove: true}}` removed the inherited choice"
    );
    assert_eq!(
        names_under(&document, "pages/partners.list/header/actions"),
        ["create", "import", "export"]
    );
    let page = &document.pages["partners.list"];
    let header = page
        .header
        .as_ref()
        .expect("the kind's header is inherited");
    assert_eq!(header.title.as_deref(), Some("Partners"));
    assert_eq!(header.actions[0].label.as_deref(), Some("New partner"));
    assert!(
        page.state.contains_key("search"),
        "list_page state is inherited"
    );
    assert!(
        page.state.contains_key("tags"),
        "entity_list_page state is inherited"
    );
    let create = &page.overlays["create"];
    assert_eq!(create.kind, ess_ui::OverlayKind::Drawer);
    let Body::Composite(Composite::Form(form)) = &create.body else {
        panic!("a form: {create:?}")
    };
    assert_eq!(form.does, "partners.CreatePartner");
    let saved = &document.pages["deals.saved"];
    assert_eq!(
        names_under(&document, "pages/deals.saved/sections"),
        ["list"]
    );
    assert_eq!(
        saved.header.as_ref().and_then(|h| h.filters.as_deref()),
        None
    );
}

#[test]
fn a_property_of_another_member_is_refused_on_this_member() {
    let mut document = example_value();
    set(
        &mut document,
        &["pages", "overview", "sections", "kpis", "item", "revenue"],
        "columns",
        entry("[name]"),
    );
    let error = refused(&document);
    assert_eq!(
        error.path().to_string(),
        "pages/overview/sections/kpis/item/revenue"
    );
    assert!(error.message().contains("columns"), "{error}");
}
