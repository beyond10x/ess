//! Three presentation keys (story:choices-filter-bars-and-forms-render-what-they-declare,
//! decisions G-13, G-14 and G-15): `placeholder` on a choice, `reset: true` or `reset: {label}` on
//! a filter bar, and `help: {text, link}` on a form group and on a field, a column included.

use ess_ui::{Body, Composite, Document, Field, FormGroup, NodeRef};

const BASE: &str = r"format: ess-ui/1
app: shelf
model: shelf.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: Items
    state:
      stage: {type: string, class: page_state, store: memory}
    sections:
";

fn text(sections: &str) -> String {
    format!("{BASE}{sections}")
}

fn load(sections: &str) -> Document {
    let text = text(sections);
    ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn located<'a>(document: &'a Document, path: &str) -> NodeRef<'a> {
    document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at `{path}`"))
        .node
}

fn bar(document: &Document) -> &ess_ui::FilterBar {
    match located(document, "pages/p/sections/filters") {
        NodeRef::Section(section) => match &section.body {
            Body::Composite(Composite::FilterBar(bar)) => bar,
            other => panic!("not a filter bar: {other:?}"),
        },
        other => panic!("not a section: {other:?}"),
    }
}

fn filter_bar(reset: &str) -> String {
    format!(
        "      - name: filters
        component: filter_bar
        binds: [state.stage]
        choices:
          - {{name: stage, component: choice, options: [open, won], placeholder: Any stage}}
{reset}"
    )
}

#[test]
fn a_choice_takes_a_placeholder() {
    let document = load(&filter_bar(""));
    let NodeRef::Node(node) = located(&document, "pages/p/sections/filters/choices/stage") else {
        panic!("the choice is a node");
    };
    let Body::Composite(Composite::Choice(choice)) = &node.body else {
        panic!("the node is a choice");
    };
    assert_eq!(choice.placeholder.as_deref(), Some("Any stage"));
}

#[test]
fn reset_is_true_or_a_label() {
    let absent = load(&filter_bar(""));
    assert!(!bar(&absent).offers_reset());
    assert_eq!(bar(&absent).reset_label(), None);

    let off = load(&filter_bar("        reset: false"));
    assert!(!bar(&off).offers_reset());

    let on = load(&filter_bar("        reset: true"));
    assert!(bar(&on).offers_reset());
    assert_eq!(bar(&on).reset_label(), None);

    let labelled = load(&filter_bar("        reset: {label: Clear filters}"));
    assert!(bar(&labelled).offers_reset());
    assert_eq!(bar(&labelled).reset_label(), Some("Clear filters"));
}

#[test]
fn a_reset_with_another_key_or_no_label_is_refused() {
    for reset in [
        "        reset: {label: Clear, tone: danger}",
        "        reset: {}",
        "        reset: Clear",
    ] {
        let text = text(&filter_bar(reset));
        assert!(
            ess_ui::load_str(&text).is_err(),
            "`{}` is refused",
            reset.trim()
        );
    }
}

#[test]
fn help_is_read_on_a_form_group_a_field_and_a_column() {
    let document = load(
        "      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns:
          - {field: title, help: {text: The name shown}}
      - name: form
        component: form
        does: shelf.Configure
        groups:
          - name: profile
            help: {text: Who sees it, link: 'https://example.test/profile'}
            fields:
              - {field: owner, help: {link: 'https://example.test/owner'}}
",
    );
    let field = |path: &str| -> &Field {
        match located(&document, path) {
            NodeRef::Field(field) => field,
            other => panic!("not a field at `{path}`: {other:?}"),
        }
    };
    let column = field("pages/p/sections/list/columns/title");
    let help = column.help.as_ref().expect("the column has help");
    assert_eq!(help.text.as_deref(), Some("The name shown"));
    assert_eq!(help.link, None);

    let owner = field("pages/p/sections/form/groups/profile/fields/owner");
    let help = owner.help.as_ref().expect("the field has help");
    assert_eq!(help.link.as_deref(), Some("https://example.test/owner"));

    let NodeRef::FormGroup(group) = located(&document, "pages/p/sections/form/groups/profile")
    else {
        panic!("the group is a form group");
    };
    let group: &FormGroup = group;
    let help = group.help.as_ref().expect("the group has help");
    assert_eq!(help.text.as_deref(), Some("Who sees it"));
    assert_eq!(help.link.as_deref(), Some("https://example.test/profile"));
}

#[test]
fn help_takes_no_other_key() {
    let text = text(
        "      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns:
          - {field: title, help: {text: The name, tone: info}}
",
    );
    assert!(
        ess_ui::load_str(&text).is_err(),
        "an unknown help key is refused"
    );
}

#[test]
fn the_schema_declares_the_three_keys() {
    let schema: ess_yaml::Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
    let constructs = &schema["constructs"];
    assert!(
        constructs["choice"]["fields"].get("placeholder").is_some(),
        "choice declares `placeholder`"
    );
    let reset = format!("{:?}", constructs["filter_bar"]["fields"]["reset"]["type"]);
    assert!(reset.contains("label"), "reset takes a label: {reset}");
    for construct in ["FormGroup", "Field"] {
        assert!(
            constructs[construct]["fields"].get("help").is_some(),
            "{construct} declares `help`"
        );
    }
}
