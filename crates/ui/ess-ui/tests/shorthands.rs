//! One instance of every shorthand in the schema's `shorthands.index` expands as its
//! `expands_to` says.
//!
//! The cases are keyed by the index's own `(construct, at)`, and the first test fails when the
//! schema gains a shorthand this file has no case for, or loses one it has.

use std::collections::BTreeSet;

use ess_ui::{
    ActionConfirm, Body, ChoiceOption, Columns, Composite, Document, Node, OverlayKind, Page,
    PageLayout, Section,
};
use serde_yaml::Value;

const DOCUMENT: &str = r"
format: ess-ui/1
app: things
model: things.system
placement_profile: fat
types:
  Stage: {enum: [lead, won]}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.detail]}]
page_kinds:
  thing_list_page:
    extends: list_page
    sections:
      - name: filters
        choices:
          - {name: stage, component: choice, options: Stage}
          - {name: kind, component: choice, options: [a, b]}
          - {name: extra, component: choice, options: [x]}
      - {name: notes, component: record}
    overlays:
      edit: {kind: drawer, component: form, does: things.UpdateThing, fields: [title]}
    header: {actions: [{name: create, opens: edit, label: New}]}
pages:
  things.list:
    kind: thing_list_page
    title: Things
    nav: {synonyms: [stuff]}
    state:
      open: {type: boolean, class: component_state}
    header: {filters: null}
    sections:
      - name: filters
        choices:
          - {name: extra, remove: true}
      - {name: notes, remove: true}
      - name: list
        reads: things.Page
        columns: [title, {field: stage, as: badge}]
        expand: record
        row_actions:
          - {does: things.ArchiveThing, confirm: {title: Archive this thing}}
          - {sets: {state.open: toggle}}
  things.detail:
    kind: detail_page
    title: Thing
    overlays:
      edit: {kind: drawer, component: form, same_as: things.list.edit, title: Edit thing}
    sections:
      - name: summary
        reads: {view: things.ById}
";

fn document() -> Document {
    ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"))
}

fn page<'a>(document: &'a Document, name: &str) -> &'a Page {
    document.pages.get(name).expect("the page exists")
}

fn section<'a>(page: &'a Page, name: &str) -> &'a Section {
    page.sections
        .iter()
        .find(|section| section.name == name)
        .unwrap_or_else(|| panic!("no section `{name}`"))
}

fn composite(body: &Body) -> &Composite {
    match body {
        Body::Composite(composite) => composite,
        other => panic!("not a composite: {other:?}"),
    }
}

fn list(document: &Document) -> &ess_ui::Collection {
    match composite(&section(page(document, "things.list"), "list").body) {
        Composite::Collection(collection) => collection,
        other => panic!("not a collection: {other:?}"),
    }
}

fn filter_choices(document: &Document) -> &[Node] {
    match composite(&section(page(document, "things.list"), "filters").body) {
        Composite::FilterBar(bar) => &bar.choices,
        other => panic!("not a filter bar: {other:?}"),
    }
}

fn choice_options<'a>(choices: &'a [Node], name: &str) -> &'a [ChoiceOption] {
    let node = choices
        .iter()
        .find(|node| node.common.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no choice `{name}`"));
    match composite(&node.body) {
        Composite::Choice(choice) => &choice.options,
        other => panic!("not a choice: {other:?}"),
    }
}

fn options(pairs: &[(&str, &str)]) -> Vec<ChoiceOption> {
    pairs
        .iter()
        .map(|(value, label)| ChoiceOption {
            value: Value::from(*value),
            label: (*label).to_owned(),
        })
        .collect()
}

type Case = fn(&Document);

#[allow(clippy::too_many_lines)] // one case per shorthand of the schema index
fn cases() -> Vec<((&'static str, &'static str), Case)> {
    vec![
        (("Field", ""), |document| {
            let Some(Columns::Fixed(columns)) = &list(document).columns else {
                panic!("fixed columns")
            };
            assert_eq!(
                (columns[0].field.as_str(), columns[0].name.as_str()),
                ("title", "title")
            );
        }),
        (("Field", "name"), |document| {
            let Some(Columns::Fixed(columns)) = &list(document).columns else {
                panic!("fixed columns")
            };
            assert_eq!(columns[1].name, "stage");
        }),
        (("Reads", ""), |document| {
            let reads = list(document).reads.as_ref().expect("the list reads");
            assert_eq!(reads.view.as_deref(), Some("things.Page"));
        }),
        (("Composite", ""), |document| {
            let expand = list(document).expand.as_ref().expect("rows expand");
            assert!(matches!(composite(&expand.body), Composite::Record(_)));
        }),
        (("choice", "options[]"), |document| {
            assert_eq!(
                choice_options(filter_choices(document), "kind"),
                options(&[("a", "a"), ("b", "b")])
            );
        }),
        (("choice", "options"), |document| {
            assert_eq!(
                choice_options(filter_choices(document), "stage"),
                options(&[("lead", "lead"), ("won", "won")])
            );
        }),
        (("header", "title"), |document| {
            let header = page(document, "things.list")
                .header
                .as_ref()
                .expect("a header");
            assert_eq!(header.title.as_deref(), Some("Things"));
        }),
        (("Action", "sets.*"), |document| {
            let action = &list(document).row_actions[1];
            assert_eq!(action.sets["state.open"].0, "not state.open");
        }),
        (("Action", "confirm"), |document| {
            let action = &list(document).row_actions[0];
            let Some(ActionConfirm::Inline(inline)) = &action.confirm else {
                panic!("an inline confirm: {action:?}")
            };
            assert_eq!(inline.overlay.kind, OverlayKind::Dialog);
            assert_eq!(inline.overlay.title.as_deref(), Some("Archive this thing"));
            let Composite::Confirm(confirm) = composite(&inline.overlay.body) else {
                panic!("a confirm")
            };
            assert_eq!(confirm.does.as_deref(), Some("things.ArchiveThing"));
        }),
        (("Action", "name"), |document| {
            let names: Vec<&str> = list(document)
                .row_actions
                .iter()
                .map(|action| action.name.as_str())
                .collect();
            assert_eq!(names, ["archive_thing", "state.open"]);
        }),
        (("Page", "state.* | overlays.* | header.*"), |document| {
            let header = page(document, "things.list")
                .header
                .as_ref()
                .expect("a header");
            assert_eq!(
                header.filters, None,
                "the inherited `filters: filters` is removed"
            );
            assert_eq!(
                header.total.as_deref(),
                Some("list"),
                "the rest is inherited"
            );
        }),
        (("Page", "sections[]"), |document| {
            let names: Vec<&str> = page(document, "things.list")
                .sections
                .iter()
                .map(|section| section.name.as_str())
                .collect();
            assert_eq!(names, ["filters", "list"]);
        }),
        (("filter_bar", "choices[]"), |document| {
            let names: Vec<&str> = filter_choices(document)
                .iter()
                .map(|node| node.common.name.as_deref().unwrap_or(""))
                .collect();
            assert_eq!(names, ["stage", "kind"]);
        }),
        (("Page", "shell"), |document| {
            assert_eq!(page(document, "things.list").shell, "app");
        }),
        (("overlay", "same_as"), |document| {
            let overlay = &page(document, "things.detail").overlays["edit"];
            assert_eq!(overlay.title.as_deref(), Some("Edit thing"));
            assert_eq!(overlay.same_as.as_deref(), Some("things.list.edit"));
            let Composite::Form(form) = composite(&overlay.body) else {
                panic!("a form")
            };
            assert_eq!(form.does, "things.UpdateThing");
            assert_eq!(form.fields[0].name, "title");
        }),
        (("NavEntry", "label"), |document| {
            let nav = page(document, "things.list")
                .nav
                .as_ref()
                .expect("a nav entry");
            assert_eq!(nav.label.as_deref(), Some("Things"));
        }),
        (("PageLayout", ""), |document| {
            assert!(matches!(
                page(document, "things.list").layout,
                PageLayout::Stack(_)
            ));
        }),
    ]
}

fn index_keys() -> BTreeSet<(String, String)> {
    let schema: Value = serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
    schema["shorthands"]["index"]
        .as_sequence()
        .expect("the shorthand index is a list")
        .iter()
        .map(|entry| {
            (
                entry["construct"].as_str().expect("a construct").to_owned(),
                entry["at"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect()
}

#[test]
fn every_shorthand_in_the_schema_index_has_a_case() {
    let covered: BTreeSet<(String, String)> = cases()
        .iter()
        .map(|((construct, at), _)| ((*construct).to_owned(), (*at).to_owned()))
        .collect();
    assert_eq!(covered, index_keys());
}

#[test]
fn one_instance_of_every_shorthand_expands_as_the_schema_says() {
    let document = document();
    for (_, case) in cases() {
        case(&document);
    }
}

#[test]
fn a_widget_use_without_a_required_argument_is_refused() {
    let text = DOCUMENT.replace(
        "types:\n",
        "widgets:\n  pill:\n    summary: A pill.\n    params: {text: {type: string, required: true, note: text}}\n    body: [{name: badge, primitive: badge, text: args.text}]\ntypes:\n",
    )
    .replace(
        "        expand: record\n",
        "        expand: record\n        item: [{name: tag, component: pill, args: {}}]\n",
    );
    let error = ess_ui::load_str(&text).expect_err("a missing argument is refused");
    assert_eq!(
        error.path().to_string(),
        "pages/things.list/sections/list/item/tag"
    );
    assert!(error.message().contains("text"), "{error}");
}

/// Every position the schema types `Node`, with its shape: the `Composite` shorthand is applied at
/// each. A position the schema gains fails this list until a case below covers it.
#[test]
fn the_composite_shorthand_applies_at_every_node_position_of_the_schema() {
    use ess_ui::Shape::{List, Map, One};
    let found: BTreeSet<(String, ess_ui::Shape)> = ess_ui::positions().nodes.clone();
    let expected: BTreeSet<(String, ess_ui::Shape)> = [
        ("expand", One),
        ("choice", One),
        ("result", One),
        ("record", One),
        ("form", One),
        ("item", List),
        ("children", List),
        ("body", List),
        ("parts", List),
        ("choices", List),
        ("metrics", List),
        ("toolbar", List),
        ("widgets", Map),
    ]
    .into_iter()
    .map(|(key, shape)| (key.to_owned(), shape))
    .collect();
    assert_eq!(found, expected);

    let text = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - {name: summary, reads: t.ById, tabs: [{name: more, form: record}]}
      - name: list
        component: collection
        reads: t.Page
        expand: record
        columns: [{field: stage, as: choice, choice: choice}]
        row_actions: [{does: t.Move, as: choice, choice: choice}]
      - {name: form, component: form, does: t.Save, result: record, record: record}
      - {name: board, component: board, reads: t.Board, widgets: {rate: metric}}
";
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let composites: BTreeSet<String> = document
        .nodes()
        .into_iter()
        .filter(|located| {
            matches!(located.node, ess_ui::NodeRef::Node(node) if matches!(node.body, Body::Composite(_)))
        })
        .map(|located| located.path.to_string())
        .collect();
    for path in [
        "pages/p/sections/summary/tabs/more/form",
        "pages/p/sections/list/expand",
        "pages/p/sections/list/columns/stage/choice",
        "pages/p/sections/list/row_actions/move/choice",
        "pages/p/sections/form/result",
        "pages/p/sections/form/record",
        "pages/p/sections/board/widgets/rate",
    ] {
        assert!(
            composites.contains(path),
            "no composite at `{path}`: {composites:?}"
        );
    }

    // In a list, the shorthand yields a node with no name, which a list refuses.
    let listed = text.replace("expand: record", "expand: record\n        item: [record]");
    let error = ess_ui::load_str(&listed).expect_err("an unnamed list node is refused");
    assert!(
        error
            .path()
            .to_string()
            .starts_with("pages/p/sections/list/item"),
        "{error}"
    );
}

/// Every short form is checked against its `accepts` before it expands (`shorthands.validation`).
#[test]
fn a_short_form_its_accepts_does_not_admit_is_refused_with_its_path() {
    let confirm = DOCUMENT.replace(
        "confirm: {title: Archive this thing}",
        "confirm: {title: Archive this thing, colour: red}",
    );
    let error = ess_ui::load_str(&confirm).expect_err("an unknown confirm key is refused");
    assert_eq!(
        error.path().to_string(),
        "pages/things.list/sections/list/row_actions/confirm",
        "{error}"
    );
    assert!(
        error.message().contains("Action confirm shorthand accepts"),
        "{error}"
    );

    let widget = DOCUMENT.replace("expand: record", "expand: pill");
    let error = ess_ui::load_str(&widget).expect_err("a bare non-member name is refused");
    assert_eq!(
        error.path().to_string(),
        "pages/things.list/sections/list/expand",
        "{error}"
    );
    assert!(
        error.message().contains("{component: <widget>, args: …}"),
        "{error}"
    );
}

/// A marker written as a `reads` is no read; the section records it among its gaps.
#[test]
fn an_unmapped_marker_at_reads_is_recorded_as_a_gap() {
    let text = DOCUMENT.replace(
        "reads: {view: things.ById}",
        "reads: 'UNMAPPED: built at runtime'",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let summary = section(page(&document, "things.detail"), "summary");
    let Composite::Record(record) = composite(&summary.body) else {
        panic!("a record")
    };
    assert_eq!(record.reads, None);
    assert_eq!(
        summary.common.unmapped,
        ["reads: UNMAPPED: built at runtime"]
    );
}

#[test]
fn a_removal_entry_with_anything_beside_name_and_remove_is_refused() {
    let text = DOCUMENT.replace(
        "{name: notes, remove: true}",
        "{name: notes, remove: true, component: record}",
    );
    let error = ess_ui::load_str(&text).expect_err("a removal with extra keys is refused");
    assert_eq!(
        error.path().to_string(),
        "pages/things.list/sections/notes",
        "{error}"
    );
}

/// beyond10x/ess#330: `options` naming an enum of the ESS model is refused, and the refusal says
/// why (a renderer has no model) and what to write instead.
#[test]
fn options_naming_a_model_enum_say_what_to_write_instead() {
    let text = DOCUMENT.replace(
        "{name: stage, component: choice, options: Stage}",
        "{name: stage, component: choice, options: factory.objective.RiskLevel}",
    );
    let error = ess_ui::load_str(&text).expect_err("a model enum has no variants here");
    assert!(error.message().contains("`types`"), "{error}");
    assert!(error.message().contains("--model"), "{error}");
}

/// A model as `ess ui check --model` gives one to the loader: `factory.objective.RiskLevel` is an
/// enum whose wire spellings differ from its display names, `factory.objective.ObjectiveId` is a
/// type that is not one, `Level` could be either of two enums, and `Stage` is an enum the
/// document also declares.
struct Factory;

impl ess_ui::binding::ModelEnums for Factory {
    fn system(&self) -> &'static str {
        "factory"
    }

    fn lookup(&self, name: &str) -> ess_ui::binding::EnumLookup {
        use ess_ui::binding::{EnumLookup, EnumVariant};
        let variant = |value: &str, label: &str| EnumVariant {
            value: value.to_owned(),
            label: label.to_owned(),
        };
        match name {
            "factory.objective.RiskLevel" | "objective.RiskLevel" => EnumLookup::Enum {
                name: "factory.objective.RiskLevel".to_owned(),
                variants: vec![
                    variant("low", "Low"),
                    variant("medium", "Medium"),
                    variant("high", "High"),
                ],
            },
            "Stage" => EnumLookup::Enum {
                name: "factory.objective.Stage".to_owned(),
                variants: vec![variant("model-stage", "Model stage")],
            },
            "factory.objective.ObjectiveId" => EnumLookup::NotEnum {
                name: "factory.objective.ObjectiveId".to_owned(),
            },
            "Level" => EnumLookup::Ambiguous(vec![
                "factory.objective.Level".to_owned(),
                "factory.release.Level".to_owned(),
            ]),
            _ => EnumLookup::Unknown,
        }
    }
}

fn with_stage_options(options: &str) -> String {
    DOCUMENT.replace(
        "{name: stage, component: choice, options: Stage}",
        &format!("{{name: stage, component: choice, options: {options}}}"),
    )
}

/// beyond10x/ess#330: with a model, `options` naming one of its enums lists its variants in
/// declaration order, each sending its wire spelling and showing its display name, in a
/// standalone choice and in a form field's; the document records which name resolved where.
#[test]
fn options_naming_a_model_enum_list_its_variants_when_the_model_is_given() {
    let text = with_stage_options("factory.objective.RiskLevel").replace(
        "fields: [title]}",
        "fields: [title, {field: risk, as: choice, choice: {component: choice, options: objective.RiskLevel}}]}",
    );
    let document = ess_ui::load_str_with(&text, &Factory).unwrap_or_else(|error| panic!("{error}"));
    let expected = options(&[("low", "Low"), ("medium", "Medium"), ("high", "High")]);
    assert_eq!(
        choice_options(filter_choices(&document), "stage"),
        expected.as_slice()
    );
    let overlay = &page(&document, "things.list").overlays["edit"];
    let Composite::Form(form) = composite(&overlay.body) else {
        panic!("a form")
    };
    let field = form
        .fields
        .iter()
        .find(|field| field.field == "risk")
        .expect("the risk field");
    let Some(Body::Composite(Composite::Choice(choice))) =
        field.choice.as_deref().map(|node| &node.body)
    else {
        panic!("a choice field")
    };
    assert_eq!(choice.options, expected);
    assert_eq!(
        document.model_enums,
        [
            ("factory.objective.RiskLevel", "factory.objective.RiskLevel"),
            ("objective.RiskLevel", "factory.objective.RiskLevel"),
        ]
        .into_iter()
        .map(|(written, qualified)| (written.to_owned(), qualified.to_owned()))
        .collect()
    );
}

/// An enum the document declares under `types` is read from the document, as it is without a
/// model: the model is asked only for a name the document does not declare.
#[test]
fn a_document_enum_wins_over_a_model_enum_of_the_same_name() {
    let document =
        ess_ui::load_str_with(DOCUMENT, &Factory).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        choice_options(filter_choices(&document), "stage"),
        options(&[("lead", "lead"), ("won", "won")]).as_slice()
    );
    assert!(document.model_enums.is_empty());
    assert_eq!(document, self::document(), "the model changes nothing else");
}

/// A name the model does not resolve to exactly one enum is refused at the options, each kind
/// of failure in its own words; so is any model name when no model is given.
#[test]
fn options_the_model_does_not_resolve_to_one_enum_are_refused_by_kind() {
    let at = "pages/things.list/sections/filters/choices/stage/options";
    for (written, model, says) in [
        (
            "factory.objective.ObjectiveId",
            true,
            "which is not an enum",
        ),
        (
            "Level",
            true,
            "`factory.objective.Level`, `factory.release.Level`",
        ),
        (
            "factory.objective.Nothing",
            true,
            "names no type of model `factory`",
        ),
        ("factory.objective.RiskLevel", false, "no model was given"),
    ] {
        let text = with_stage_options(written);
        let error = if model {
            ess_ui::load_str_with(&text, &Factory)
        } else {
            ess_ui::load_str(&text)
        }
        .expect_err(written);
        assert_eq!(error.path().to_string(), at, "{written}: {error}");
        assert!(
            error.message().starts_with("choice options: "),
            "{written}: {error}"
        );
        assert!(error.message().contains(says), "{written}: {error}");
        assert!(
            error.message().contains(&format!("`{written}`")),
            "{written}: {error}"
        );
    }
}

/// A renderer holds no model, only the binding `--model` computed: the binding resolves the
/// enums it carries, by the name the document writes, and nothing else.
#[test]
fn a_binding_resolves_the_model_enums_it_carries() {
    use ess_ui::binding::{Binding, EnumVariant};
    let binding: Binding = serde_json::from_str(
        r#"{
  "system": "factory",
  "components": {},
  "names": {"objective.RiskLevel": "factory.objective.RiskLevel"},
  "enums": {"factory.objective.RiskLevel": [{"value": "low", "label": "Low"}, {"value": "high", "label": "High"}]}
}"#,
    )
    .expect("the binding reads");
    assert_eq!(
        binding.enums["factory.objective.RiskLevel"],
        [
            EnumVariant {
                value: "low".to_owned(),
                label: "Low".to_owned()
            },
            EnumVariant {
                value: "high".to_owned(),
                label: "High".to_owned()
            }
        ]
    );
    let document = ess_ui::load_str_with(&with_stage_options("objective.RiskLevel"), &binding)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        choice_options(filter_choices(&document), "stage"),
        options(&[("low", "Low"), ("high", "High")]).as_slice()
    );
    let error = ess_ui::load_str_with(&with_stage_options("objective.Other"), &binding)
        .expect_err("an enum the binding does not carry");
    assert!(
        error.message().contains("names no type of model `factory`"),
        "{error}"
    );
}
